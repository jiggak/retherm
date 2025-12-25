/*
 * Nest UI - Home Assistant native thermostat interface
 * Copyright (C) 2025 Josh Kropf <josh@slashdev.ca>
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

use std::sync::mpsc::{Receiver, Sender, channel};

use anyhow::Result;

// If I understand correctly, I don't want to have a method in EventPump
// for injecting events, but rather clone the sender of the channel into
// the various threads that produce events.

#[derive(Debug)]
pub enum Event {
    ButtonDown,
    Dial(i32),
    Temp,
    HVAC
}

impl Event {
    /// Returns true if the event is one of the types that should cause device wakeup
    pub fn is_wakeup_event(&self) -> bool {
        match self {
            Event::ButtonDown | Event::Dial(_) => true,
            _ => false
        }
    }
}

pub struct EventPump {
    pub sender: Sender<Event>,
    receiver: Receiver<Event>
}

impl EventPump {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }

    pub fn wait_event(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }
}