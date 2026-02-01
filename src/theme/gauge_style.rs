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

use embedded_graphics::pixelcolor::Bgr888;
use serde::Deserialize;

use super::{theme_de, FontDef};

#[derive(Deserialize, Clone)]
pub struct GaugeStyle {
    #[serde(deserialize_with = "theme_de::colour")]
    pub fg_colour: Bgr888,

    /// Diameter of guage arch
    pub arc_dia: u32,
    /// Width of arc
    pub arc_width: u32,
    /// Arc start angle; 0 degrees at 3'oclock
    pub arc_start_deg: f32,
    pub arc_sweed_deg: f32,

    /// Target temp decimal digit font
    pub target_font: FontDef<'static>,
    /// Target temp fraction digit font
    pub target_decimal_font: FontDef<'static>,
    /// Current temp font
    pub current_font: FontDef<'static>,

    #[serde(deserialize_with = "theme_de::colour")]
    pub arc_bg_colour: Bgr888,

    #[serde(deserialize_with = "theme_de::colour")]
    pub arc_heat_colour: Bgr888,
    #[serde(deserialize_with = "theme_de::colour")]
    pub arc_heat_dot_colour: Bgr888,

    #[serde(deserialize_with = "theme_de::colour")]
    pub arc_cool_colour: Bgr888,
    #[serde(deserialize_with = "theme_de::colour")]
    pub arc_cool_dot_colour: Bgr888,

    /// Diameter of target temp dot
    pub arc_target_dot_dia: u32,

    /// Current temp dot diameter
    pub arc_temp_dot_dia: u32,
    /// Current temp dot colour
    #[serde(deserialize_with = "theme_de::colour")]
    pub arc_temp_dot_colour: Bgr888,
    /// Current temp label
    pub arc_temp_text_dia: u32
}
