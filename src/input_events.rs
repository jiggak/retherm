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

use std::thread::{self, JoinHandle};

use anyhow::{Result, anyhow};
use evdev::{Device, EventSummary, KeyCode};

use crate::events::{Event, EventSender};

struct InputDevice {
    device: Device,
    map_fn: InputEventMapFn
}

type InputEventMapFn = fn(EventSummary) -> Option<Event>;

impl InputDevice {
    fn open(path: &str, map_fn: InputEventMapFn) -> Result<Self> {
        let device = Device::open(path)?;
        Ok(Self {
            device,
            map_fn
        })
    }

    fn fetch_events(&mut self) -> Result<impl Iterator<Item = Event>> {
        let events = self.device.fetch_events()?
            .filter_map(|event| {
                (self.map_fn)(event.destructure())
            });
        Ok(events)
    }
}

pub struct InputDeviceThread {
    thread: JoinHandle<Result<()>>
}

impl InputDeviceThread {
    fn start<S>(mut input_events: InputDevice, sender: S) -> Self
        where S: EventSender + Send + 'static
    {
        let thread = thread::spawn(move || {
            loop {
                let events = input_events.fetch_events()?;
                for event in events {
                    sender.send_event(event)?;
                }
            }
        });

        Self {
            thread
        }
    }

    pub fn stop(self) -> Result<()> {
        // FIXME find some way to close input event device so fetch_events() stops blocking
        let handle = self.thread.join()
            .map_err(|e| anyhow!("{:?}", e))?;
        handle
    }
}

pub fn start_dial_events<S>(sender: S) -> Result<InputDeviceThread>
    where S: EventSender + Send + 'static
{
    let input_events = InputDevice::open(
        "/dev/input/event1",
        |e| match e {
            // value > 0 = counter clockwise, value < 0 clockwise
            EventSummary::RelativeAxis(_, _, value) => {
                // invert value so clockwise is increasing
                Some(Event::Dial(value * -1))
            }
            _ => None
        }
    )?;

    Ok(InputDeviceThread::start(input_events, sender))
}

pub fn start_button_events<S>(sender: S) -> Result<InputDeviceThread>
    where S: EventSender + Send + 'static
{
    let input_events = InputDevice::open(
        "/dev/input/event2",
        |e| match e {
            // value 1 = down, followed by value 0 = up
            EventSummary::Key(_, KeyCode::KEY_POWER, 1) => {
                Some(Event::ButtonDown)
            }
            _ => None
        }
    )?;

    Ok(InputDeviceThread::start(input_events, sender))
}
