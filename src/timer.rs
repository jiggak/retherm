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

use std::{collections::HashMap, sync::mpsc::{Sender, channel}, thread, time::Duration};

use crate::events::{Event, EventHandler, EventSender};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimerId {
    Away,
    // Backlight,
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
                Ok(new_timeout) => {
                    timeout = new_timeout;
                }
                Err(_) => {
                    break;
                }
            }
            if receiver.recv_timeout(timeout).is_err() {
                break;
            }
        }

        timeout_reached();
    });

    sender
}

pub struct Timers<S> {
    timers: HashMap<TimerId, Sender<Duration>>,
    event_sender: S
}

impl<S: EventSender + Clone + Send + 'static> Timers<S> {
    pub fn new(event_sender: S) -> Self {
        Self {
            timers: HashMap::new(),
            event_sender
        }
    }

    fn start_timer(&self, id: TimerId, timeout: Duration) -> Sender<Duration> {
        let timeout_sender = self.event_sender.clone();

        start_timeout_thread(timeout, move || {
            timeout_sender.send_event(Event::TimeoutReached(id)).unwrap();
        })
    }
}

impl<S: EventSender + Clone + Send + 'static> EventHandler for Timers<S> {
    fn handle_event(&mut self, event: &Event) -> anyhow::Result<()> {
        match *event {
            Event::TimeoutReached(id) => {
                self.timers.remove(&id);
            }
            Event::TimeoutReset(id, timeout) => {
                if let Some(sender) = self.timers.get(&id) {
                    sender.send(timeout).unwrap();
                } else {
                    self.timers.insert(id, self.start_timer(id, timeout));
                }
            }
            _ => { }
        }
        Ok(())
    }
}
