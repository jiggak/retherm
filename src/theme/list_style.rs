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

/// Mode select list style
#[derive(Deserialize, Clone)]
pub struct ListStyle {
    /// Colour of list item text, default "#d3d3d3"
    #[serde(deserialize_with = "theme_de::colour")]
    pub colour: Bgr888,

    /// List item font, default "Bold:36"
    pub label_font: FontDef<'static>,

    /// Selected item icon font, default "Icon:20"
    pub icon_font: FontDef<'static>,

    /// Selected item icon, default "\u{f00c}"
    pub selected_icon: String,

    /// Highlighted row text colour, default "#ffffff"
    #[serde(deserialize_with = "theme_de::colour")]
    pub highlight_text_colour: Bgr888,

    /// Style of the highlight row, default `{ fill_colour: "#", corner_radius: 18 }`
    pub highlight_rect: RectStyle,

    /// List item row size, default `[140, 40]`
    #[serde(deserialize_with = "theme_de::size")]
    pub row_size: Size
}
