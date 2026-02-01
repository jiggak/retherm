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

use embedded_graphics::prelude::PixelColor;
use embedded_ttf::{FontTextStyle, FontTextStyleBuilder};
use rusttype::Font;

#[derive(Clone, Debug)]
pub struct FontDef<'a> {
    pub font: Font<'a>,
    pub size: u32
}

impl<'a> FontDef<'a> {
    pub fn new(font: &Font<'a>, size: u32) -> Self {
        Self { font: font.clone(), size }
    }
}

impl<'a> FontDef<'a> where 'a: 'static {
    pub fn font_style<C: PixelColor>(&self, fg_color: C, bg_color: C) -> FontTextStyle<C> {
        FontTextStyleBuilder::new(self.font.clone())
            .font_size(self.size)
            .text_color(fg_color)
            .anti_aliasing_color(bg_color)
            .build()
    }
}
