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

use std::{cell::RefCell, sync::mpsc::{Receiver, Sender, channel}, time::Duration};

use anyhow::Result;
use throttle::Throttle;

use crate::backplate::{HvacMode, HvacState};

#[derive(Debug)]
pub enum Event {
    ButtonDown,
    Dial(i32),
    SetTargetTemp(f32),
    SetMode(HvacMode),
    HvacState(HvacState),
    Quit
}

impl Event {
    /// Returns true if the event is one of the types that should cause device wakeup
    pub fn is_wakeup_event(&self) -> bool {
        match self {
            Self::ButtonDown | Self::Dial(_) => true,
            _ => false
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum EventOrigin {
    Interface,
    HomeAssistant,
    Backplate
}

pub trait EventSender {
    fn send_event(&self, event: Event) -> Result<()>;
}

pub trait EventHandler {
    fn handle_event(&mut self, event: &Event) -> Result<()>;
}

pub trait EventSource {
    fn wait_event(&mut self) -> Result<Event>;
    fn event_sender(&self) -> impl EventSender + Send + 'static;
}

pub struct DefaultEventSource {
    sender: Sender<Event>,
    receiver: Receiver<Event>
}

impl DefaultEventSource {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }
}

impl EventSource for DefaultEventSource {
    fn wait_event(&mut self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    fn event_sender(&self) -> impl EventSender + 'static {
        self.sender.clone()
    }
}

impl EventSender for Sender<Event> {
    fn send_event(&self, event: Event) -> Result<()> {
        Ok(self.send(event)?)
    }
}

pub struct ThrottledEventSender<S> {
    event_sender: S,
    throttle: RefCell<Throttle>
}

impl<S: EventSender> ThrottledEventSender<S> {
    /// Accept up to `threshold` events, every `timeout_ms`
    pub fn new(event_sender: S, timeout_ms: u64, threshold: usize) -> Self {
        let timeout = Duration::from_millis(timeout_ms);
        Self {
            event_sender,
            throttle: RefCell::new(Throttle::new(timeout, threshold))
        }
    }
}

impl<S: EventSender> EventSender for ThrottledEventSender<S> {
    fn send_event(&self, event: Event) -> Result<()> {
        if self.throttle.borrow_mut().accept().is_ok() {
            self.event_sender.send_event(event)?;
        }

        Ok(())
    }
}
