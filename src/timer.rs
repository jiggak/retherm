/*
 * ReTherm - Home Assistant native interface for Gen2 Nest thermostat
 * Copyright (C) 2026 Josh Kropf <josh@slashdev.ca>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc::{RecvTimeoutError, Sender, channel}},
    thread,
    time::Duration
};

use log::{debug, warn};

use crate::events::{Event, EventHandler, EventSender};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimerId {
    Away,
    Backlight,
    // HvacCooldown
}

fn start_timeout_thread<F>(mut timeout: Duration, timeout_reached: F) -> Sender<Duration>
    where F: Fn() + Send + 'static
{
    let (sender, receiver) = channel();

    thread::spawn(move || {
        loop {
            // recv_timeout() returns Err when timeout reached
            // using sender of the channel resets the timeout
            match receiver.recv_timeout(timeout) {
                Ok(new_timeout) => timeout = new_timeout,
                Err(RecvTimeoutError::Timeout) => {
                    timeout_reached();
                    break;
                }
                Err(RecvTimeoutError::Disconnected) => {
                    warn!("Timeout thread sender disconnected");
                    break;
                }
            }
        }
    });

    sender
}

pub struct Timers<S> {
    timers: Arc<Mutex<HashMap<TimerId, Sender<Duration>>>>,
    event_sender: S
}

impl<S: EventSender + Clone + Send + 'static> Timers<S> {
    pub fn new(event_sender: S) -> Self {
        Self {
            timers: Arc::new(Mutex::new(HashMap::new())),
            event_sender
        }
    }

    fn start_timer(&self, id: TimerId, timeout: Duration) -> Sender<Duration> {
        let timeout_sender = self.event_sender.clone();
        let timers = self.timers.clone();

        start_timeout_thread(timeout, move || {
            debug!("{:?} timeout reached", id);
            timers.lock().unwrap().remove(&id);
            timeout_sender.send_event(Event::TimeoutReached(id)).unwrap();
        })
    }
}

impl<S: EventSender + Clone + Send + 'static> EventHandler for Timers<S> {
    fn handle_event(&mut self, event: &Event) -> anyhow::Result<()> {
        match *event {
            Event::TimeoutReset(id, timeout) => {
                if timeout > Duration::ZERO {
                    let mut timers = self.timers.lock().unwrap();
                    if let Some(sender) = timers.get(&id) {
                        sender.send(timeout).unwrap();
                    } else {
                        timers.insert(id, self.start_timer(id, timeout));
                    }
                } else {
                    warn!("Skipping timer {:?} with zero timeout", id);
                }
            }
            _ => { }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{DefaultEventSource, EventSource};

    fn setup_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .try_init();
    }

    fn start_event_loop<S, H>(
        mut event_source: S,
        mut handler: H
    ) -> std::thread::JoinHandle<()>
        where S: EventSource<Sender<Event>> + Send + 'static,
            H: EventHandler + Send + 'static
    {
        thread::spawn(move || {
            while let Ok(event) = event_source.wait_event() {
                debug!("{:?}", event);

                if event == Event::Quit {
                    break;
                }

                handler.handle_event(&event).unwrap();
            }
        })
    }

    #[test]
    #[ignore]
    /// This test was helpful in fixing a crash when timeout reset event is fired
    /// at the same time as the timeout reached event.
    /// The bug was very sensitive to timing, and the test would reproduce the
    /// crash very inconsistently.
    fn timer_reset_contention() -> anyhow::Result<()> {
        setup_logging();

        let event_source = DefaultEventSource::new();
        let timers = Timers::new(event_source.event_sender());
        let event_sender = event_source.event_sender();

        let handle = start_event_loop(event_source, timers);

        event_sender.send_event(Event::TimeoutReset(TimerId::Backlight, Duration::from_secs(1)))?;
        thread::sleep(Duration::from_millis(1000));
        event_sender.send_event(Event::TimeoutReset(TimerId::Backlight, Duration::from_secs(1)))?;

        event_sender.send_event(Event::Quit)?;
        handle.join().unwrap();

        Ok(())
    }
}
