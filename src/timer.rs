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

use log::warn;

use crate::events::{Event, EventHandler, EventSender};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimerId {
    Away,
    Backlight,
    HvacLockout,
    Fan,
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

    fn start_timeout_thread(&self, id: TimerId, timeout: Duration) {
        let (sender, receiver) = channel();

        let timers = self.timers.clone();
        let event_sender = self.event_sender.clone();

        thread::spawn(move || {
            let mut timeout = timeout;

            loop {
                // recv_timeout() returns Err when timeout reached
                // using sender of the channel resets the timeout
                match receiver.recv_timeout(timeout) {
                    Ok(new_timeout) => timeout = new_timeout,
                    Err(RecvTimeoutError::Timeout) => {
                        timers.lock().unwrap().remove(&id);
                        event_sender.send_event(Event::TimeoutReached(id)).unwrap();
                        break;
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        warn!("Timeout thread sender disconnected");
                        break;
                    }
                }
            }
        });

        self.timers.lock().unwrap().insert(id, sender);
    }

    fn start_tick_thread(&self, id: TimerId, timeout: Duration, tick_duration: Duration) {
        fn duration_ticks(duration: Duration, tick: Duration) -> i32 {
            let ticks = duration.div_duration_f32(tick);
            ticks.round() as i32
        }

        let (sender, receiver) = channel();

        let timers = self.timers.clone();
        let event_sender = self.event_sender.clone();

        thread::spawn(move || {
            let mut ticks = 0;
            let mut timeout_ticks = duration_ticks(timeout, tick_duration);

            while ticks < timeout_ticks {
                match receiver.recv_timeout(tick_duration) {
                    Ok(new_timeout) => {
                        timeout_ticks = duration_ticks(new_timeout, tick_duration);
                        ticks = 0;
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        ticks += 1;
                        let remaining = timeout_ticks - ticks;
                        let remaining = tick_duration.mul_f32(remaining as f32);
                        event_sender.send_event(Event::TimerTick(id, remaining)).unwrap();
                    }
                    Err(RecvTimeoutError::Disconnected) => {
                        warn!("Tick thread sender disconnected");
                        return;
                    }
                }
            }

            timers.lock().unwrap().remove(&id);
            event_sender.send_event(Event::TimeoutReached(id)).unwrap();
        });

        self.timers.lock().unwrap().insert(id, sender);
    }
}

impl<S: EventSender + Clone + Send + 'static> EventHandler for Timers<S> {
    fn handle_event(&mut self, event: &Event) -> anyhow::Result<()> {
        match *event {
            Event::TimeoutReset(id, timeout) if timeout > Duration::ZERO => {
                if let Some(sender) = self.timers.lock().unwrap().get(&id) {
                    sender.send(timeout).unwrap();
                } else {
                    self.start_timeout_thread(id, timeout);
                }
            }
            Event::StartTickTimer(id, timeout) => {
                if !self.timers.lock().unwrap().contains_key(&id) {
                    let tick_duration = Duration::from_secs(1);
                    // drop fraction of second so timer ticks predictably on first iter
                    let timeout = Duration::from_secs(timeout.as_secs());
                    self.start_tick_thread(id, timeout, tick_duration);
                }
            }
            Event::CancelTimer(id) => {
                self.timers.lock().unwrap().remove(&id);
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
                log::debug!("{:?}", event);

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
