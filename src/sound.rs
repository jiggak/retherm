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

use std::{thread, time::Duration};

use anyhow::Result;
use evdev::{Device, SoundCode, SoundEvent};

pub struct Sound {
    evdev: Device
}

impl Sound {
    pub fn new() -> Result<Self> {
        let evdev = Device::open("/dev/input/event0")?;
        Ok(Self { evdev })
    }

    pub fn click(&mut self) -> Result<()> {
        self.play_beep(1)
    }

    pub fn play_beep(&mut self, duration_ms: u64) -> Result<()> {
        // I tested all the other SoundCode types
        // SND_BELL is the only one that makes a sound
        // The `value/tone` parameter to the event has no effect (to my ears)
        self.evdev.send_events(&[*SoundEvent::new(SoundCode::SND_BELL, 1)])?;
        thread::sleep(Duration::from_millis(duration_ms));
        self.evdev.send_events(&[*SoundEvent::new(SoundCode::SND_BELL, 0)])?;
        Ok(())
    }
}
