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

use std::{sync::mpsc::Sender, thread, time::Duration};

use anyhow::Result;
use evdev::{Device, EventSummary, KeyCode};
use throttle::Throttle;

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
            // Slowing down the rate of dial events has a better UX feel IMO
            // I can't help but wonder if I should apply the throttle to all
            // events, and remove the loop delay. This feels like it would be
            // bad for the CPU let it go full bore. Maybe tokio is the answer
            // with IO waiting? Maybe turn on blocking and create separate
            // threads for the two inputs?
            // I also wonder if buffering the events and dispatching an event
            // a sum of the movement in the buffer might be helpful. The event
            // consumer would know if the user moved the dial quicker.
            let mut dial_throttle = Throttle::new(Duration::from_millis(40), 1);

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
                                if dial_throttle.accept().is_ok() {
                                    sender.send(Event::Dial(value)).unwrap();
                                }
                            },
                            _ => { }
                        }
                    }
                }

                // small delay to limit CPU use
                thread::sleep(Duration::from_millis(10));
            }
        });
    }
}
