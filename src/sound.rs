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

use anyhow::Result;

use crate::events::{Event, EventHandler};

#[cfg(feature = "device")]
mod sound_evdev;

#[cfg(feature = "device")]
pub fn new_sound() -> Result<Sound<sound_evdev::SoundThread>> {
    let provider = sound_evdev::SoundThread::start("/dev/input/event0")?;
    Ok(Sound { provider })
}

#[cfg(feature = "simulate")]
mod no_sound;

#[cfg(feature = "simulate")]
pub fn new_sound() -> Result<Sound<no_sound::NoSound>> {
    let provider = no_sound::NoSound { };
    Ok(Sound { provider })
}

trait SoundProvider {
    fn click(&self) -> Result<()>;
}

pub struct Sound<P> {
    provider: P
}

impl<P: SoundProvider> EventHandler for Sound<P> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if matches!(event, Event::ClickSound) {
            self.provider.click()?;
        }

        Ok(())
    }
}
