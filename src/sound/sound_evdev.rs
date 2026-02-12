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

use std::{sync::mpsc::{Sender, channel}, thread, time::Duration};

use anyhow::Result;
use evdev::{Device, SoundCode, SoundEvent};

use super::SoundProvider;

pub struct SoundThread {
    sender: Sender<()>
}

const CLICK_DURATION: Duration = Duration::from_millis(3);
const CLICK_FREQ: i32 = 2000;

impl SoundThread {
    pub fn start(dev_path: &str) -> Result<Self> {
        let (sender, receiver) = channel();

        let mut evdev = Device::open(dev_path)?;

        // SND_BELL makes a makes a low pitch noise
        //    - `value/tone` param has no effect
        // SND_TONE matches the sound made by nlclient input events
        //    - `value/tone` param changes freq. (higher = higher pitch sound)

        thread::spawn(move || {
            while let Ok(_) = receiver.recv() {
                // sound on
                evdev.send_events(&[*SoundEvent::new(SoundCode::SND_TONE, CLICK_FREQ)])
                    .expect("Send sound on event");

                thread::sleep(CLICK_DURATION);

                // sound off
                evdev.send_events(&[*SoundEvent::new(SoundCode::SND_TONE, 0)])
                    .expect("Send sound off event");
            }
        });

        Ok(Self { sender })
    }
}

impl SoundProvider for SoundThread {
    fn click(&self) -> Result<()> {
        Ok(self.sender.send(())?)
    }
}
