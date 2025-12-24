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

use std::{sync::mpsc::Sender, thread};

use anyhow::Result;
use evdev::{Device, EventSummary, KeyCode};

use crate::event_pump::Event;

pub struct InputEvents {
    dial_input: Device,
    button_input: Device
}

impl InputEvents {
    pub fn new() -> Result<Self> {
        let dial_input = Device::open("/dev/input/event1")?;
        dial_input.set_nonblocking(true)?;

        let button_input = Device::open("/dev/input/event2")?;
        button_input.set_nonblocking(true)?;

        Ok(Self {
            dial_input, button_input
        })
    }

    pub fn start_polling(mut self, sender: Sender<Event>) {
        thread::spawn(move || {
            loop {
                // TODO handle send() error by breaking loop?

                if let Ok(events) = self.button_input.fetch_events() {
                    for e in events {
                        // println!("button event {:?}", e);
                        match e.destructure() {
                            // value 1 = down, followed by value 0 = up
                            EventSummary::Key(_, KeyCode::KEY_POWER, 1) => {
                                sender.send(Event::ButtonDown).unwrap();
                            },
                            _ => { }
                        }
                    }
                }

                if let Ok(events) = self.dial_input.fetch_events() {
                    for e in events {
                        // println!("dial event {:?}", e);
                        match e.destructure() {
                            EventSummary::RelativeAxis(_, _, value) => {

                                sender.send(Event::Dial(value)).unwrap();
                            },
                            _ => { }
                        }
                    }
                }
            }
        });
    }
}
