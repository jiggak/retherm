/*
 * Nest UI - Home Assistant native thermostat interface
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
use embedded_graphics::{pixelcolor::Bgr888, prelude::*};

use crate::{
    drawable::{AppDrawable, AppFrameBuf},
    events::{Event, EventHandler, EventSender},
    screen_manager::Screen
};

pub struct ModeScreen<S> {
    mode_list: ModeList,
    event_sender: S
}

impl<S: EventSender> ModeScreen<S> {
    pub fn new(event_sender: S) -> Self {
        Self {
            mode_list: ModeList { },
            event_sender
        }
    }
}

impl<S: EventSender> Screen for ModeScreen<S> { }

impl<S: EventSender> EventHandler for ModeScreen<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        Ok(())
    }
}

impl<S: EventSender> AppDrawable for ModeScreen<S> {
    fn draw(&self, target: &mut AppFrameBuf) -> Result<()> {
        target.clear(Bgr888::BLACK)?;

        Ok(())
    }
}

struct ModeList {

}
