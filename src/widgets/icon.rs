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

use embedded_graphics::{
    pixelcolor::Bgr888,
    prelude::{Drawable, DrawTarget, Point},
    text::{Alignment, Text}
};

use crate::theme::IconStyle;

pub struct IconWidget {
    style: IconStyle
}

impl IconWidget {
    pub fn new(style: IconStyle) -> Self {
        Self { style }
    }

    pub fn draw<D>(
        &self,
        target: &mut D,
        position: Point,
        bg_colour: Bgr888,
        fg_colour: Option<Bgr888>
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let colour = fg_colour.unwrap_or(self.style.colour);

        Text::with_alignment(
            &self.style.icon,
            position,
            self.style.icon_font.font_style(colour, bg_colour),
            Alignment::Center
        )
        .draw(target)?;

        Ok(())
    }
}
