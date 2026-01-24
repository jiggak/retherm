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
use embedded_graphics::pixelcolor::Bgr888;
use embedded_graphics_framebuf::FrameBuf;

/// Trait for screens and components drawn by screens.
pub trait AppDrawable {
    fn draw(&self, target: &mut AppFrameBuf) -> Result<()>;
}

pub type AppFrameBuf = FrameBuf<Bgr888, [Bgr888; 320 * 320]>;
