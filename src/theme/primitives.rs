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

use embedded_graphics::{pixelcolor::Bgr888, primitives::{PrimitiveStyle, PrimitiveStyleBuilder}};
use serde::Deserialize;

use super::theme_de;

#[derive(Deserialize, Clone)]
pub struct RectStyle {
    pub stroke: Option<StrokeStyle>,
    #[serde(deserialize_with = "theme_de::optional_colour")]
    pub fill_colour: Option<Bgr888>,
    pub corner_radius: u32
}

impl RectStyle {
    pub fn rect_style(&self) -> PrimitiveStyle<Bgr888> {
        let mut style_builder = PrimitiveStyleBuilder::new();

        if let Some(stroke) = &self.stroke {
            style_builder = style_builder
                .stroke_color(stroke.colour)
                .stroke_width(stroke.width);
        }

        if let Some(fill) = self.fill_colour {
            style_builder = style_builder.fill_color(fill);
        }

        style_builder.build()
    }
}

#[derive(Deserialize, Clone)]
pub struct StrokeStyle {
    pub width: u32,
    #[serde(deserialize_with = "theme_de::colour")]
    pub colour: Bgr888
}
