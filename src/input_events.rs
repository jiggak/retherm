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

pub fn start_button_events(sender: Sender<Event>) -> Result<()> {
    let mut device = Device::open("/dev/input/event2")?;

    thread::spawn(move || {
        loop {
            for e in device.fetch_events().unwrap() {
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
    });

    Ok(())
}

pub fn start_dial_events(sender: Sender<Event>) -> Result<()> {
    let mut device = Device::open("/dev/input/event1")?;

    thread::spawn(move || {
        // Slowing down the rate of dial events has a better UX feel IMO
        // I also wonder if buffering the events and dispatching an event
        // a sum of the movement in the buffer might be helpful. The event
        // consumer would know if the user moved the dial quicker.
        let mut dial_throttle = Throttle::new(Duration::from_millis(40), 1);

        loop {
            if let Ok(events) = device.fetch_events() {
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
        }
    });

    Ok(())
}
