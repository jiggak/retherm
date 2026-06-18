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

/// Main screen gauge
///
/// ```toml
/// [main_screen.gauge]
/// fg_colour = "#00ff00"
/// ```
#[derive(Deserialize, Clone)]
pub struct GaugeStyle {
    /// Colour of text, default "#ffffff"
    #[serde(deserialize_with = "theme_de::colour")]
    pub fg_colour: Bgr888,

    /// Diameter of guage arch, default 260
    pub arc_dia: u32,

    /// Width of arc, default 20
    pub arc_width: u32,
    /// Arc start angle; 0 degrees at 3'oclock, default 120
    pub arc_start_deg: f32,

    /// Sweep angle of arc, default 300
    pub arc_sweed_deg: f32,

    /// Current value label font, default "Regular:20"
    pub font: FontDef<'static>,

    /// Background fill colour of arc, default "#696969"
    #[serde(deserialize_with = "theme_de::colour")]
    pub arc_bg_colour: Bgr888,

    /// Diameter of target value dot, default 30
    pub arc_target_dot_dia: u32,

    /// Current value dot diameter, default 12
    pub arc_dot_dia: u32,

    /// Current value dot colour, default "#C0C0C0"
    #[serde(deserialize_with = "theme_de::colour")]
    pub arc_dot_colour: Bgr888,

    /// Diameter of arc current temp label position, default 220
    pub arc_text_dia: u32,
}

#[derive(Deserialize, Clone)]
pub struct GaugeAccentStyle {
    /// Fill colour of arc
    #[serde(deserialize_with = "theme_de::colour")]
    pub arc_colour: Bgr888,

    /// Target value dot colour
    #[serde(deserialize_with = "theme_de::colour")]
    pub arc_dot_colour: Bgr888,

    /// Fill the arc below or above the target value
    pub arc_fill: ArcFill,
}

#[derive(Deserialize, Clone)]
pub enum ArcFill {
    Below,
    Above,
}
