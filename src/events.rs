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

use std::{cell::RefCell, sync::mpsc::{Receiver, Sender, channel}, time::Duration};

use anyhow::Result;
use debounce::EventDebouncer;
use throttle::Throttle;

use crate::{backplate::{HvacMode, HvacState}, screen_manager::ScreenId};

#[derive(Debug)]
pub enum Event {
    ButtonDown,
    Dial(i32),
    SetTargetTemp(f32),
    SetCurrentTemp(f32),
    SetMode(HvacMode),
    HvacState(HvacState),
    NavigateTo(ScreenId),
    NavigateBack,
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

// This impl is here to support the TrailingEventSender which sends the last
// event variant after a delay (ignoring content of event).
// If this because a problem due to needing equality to include event content,
// add a new event type specifically for the TrailingEventSender with it's own
// equality impl.
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::ButtonDown => matches!(other, Self::ButtonDown),
            Self::Dial(_) => matches!(other, Self::Dial(_)),
            Self::SetTargetTemp(_) => matches!(other, Self::SetTargetTemp(_)),
            Self::SetCurrentTemp(_) => matches!(other, Self::SetCurrentTemp(_)),
            Self::SetMode(_) => matches!(other, Self::SetMode(_)),
            Self::HvacState(_) => matches!(other, Self::HvacState(_)),
            Self::NavigateTo(_) => matches!(other, Self::NavigateTo(_)),
            Self::NavigateBack => matches!(other, Self::NavigateBack),
            Self::Quit => matches!(other, Self::Quit),
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

pub trait EventSender {
    fn send_event(&self, event: Event) -> Result<()>;
}

pub trait EventHandler {
    fn handle_event(&mut self, event: &Event) -> Result<()>;
}

pub trait EventSource<S: EventSender> {
    fn wait_event(&mut self) -> Result<Event>;
    fn event_sender(&self) -> S;
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

impl EventSource<Sender<Event>> for DefaultEventSource {
    fn wait_event(&mut self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    fn event_sender(&self) -> Sender<Event> {
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

pub struct TrailingEventSender {
    event_debounce: EventDebouncer<Event>
}

impl TrailingEventSender {
    pub fn new<S>(event_sender: S, delay_ms: u64) -> Self
        where S: EventSender + Send + 'static
    {
        let delay = Duration::from_millis(delay_ms);
        let event_debounce = EventDebouncer::new(delay, move |e: Event|
            event_sender.send_event(e).unwrap()
        );
        Self { event_debounce }
    }
}

impl EventSender for TrailingEventSender {
    fn send_event(&self, event: Event) -> Result<()> {
        self.event_debounce.put(event);
        Ok(())
    }
}
