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

use embedded_graphics::{pixelcolor::Bgr888, prelude::Size};
use serde::Deserialize;

use super::{theme_de, FontDef, RectStyle};

#[derive(Deserialize, Clone)]
pub struct ListStyle {
    #[serde(deserialize_with = "theme_de::colour")]
    pub colour: Bgr888,

    pub label_font: FontDef<'static>,

    pub icon_font: FontDef<'static>,
    pub selected_icon: String,

    #[serde(deserialize_with = "theme_de::colour")]
    pub highlight_text_colour: Bgr888,
    pub highlight_rect: RectStyle,

    #[serde(deserialize_with = "theme_de::size")]
    pub row_size: Size
}
