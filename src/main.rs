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

use anyhow::Result;
use embedded_graphics::Drawable;

mod display_framebuffer;
mod main_screen;

use crate::display_framebuffer::FramebufferDisplay;
use crate::main_screen::MainScreen;

fn main() -> Result<()> {
    let mut display = FramebufferDisplay::new()?;
    let screen = MainScreen { };

    loop {
        screen.draw(&mut display.buf);
        display.flush()?;
    }
}
