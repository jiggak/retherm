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

use std::{fs, path::Path};

use anyhow::Result;
use embedded_graphics::{pixelcolor::Bgr888, prelude::*};
use serde::Deserialize;

pub use self::{
    fonts::{FontName, Fonts},
    font_def::FontDef,
    gauge_style::GaugeStyle,
    icon_style::IconStyle,
    list_style::ListStyle,
    primitives::RectStyle
};

mod font_def_de;
mod font_def;
mod fonts;
mod gauge_style;
mod icon_style;
mod list_style;
mod primitives;
mod theme_de;

/// Theme file
///
/// Launch retherm with the path to your custom theme.
///
/// ```bash
/// retherm --theme ./your_theme.toml
/// ```
///
/// All theme options have a default; you only need to include options
/// you would like to override in your theme file.
///
/// The screen size is 320x320 pixels, with the origin in the top left.
///
/// ## Fonts
///
/// Font can be specified in the format `"<name>:<size>"` where name is one
/// of the following:
///
/// * Icon: FontAwesome 7.1.0
/// * Regular: Roboto Regular
/// * Bold: Roboto Bold
#[derive(Deserialize)]
#[serde(default)]
pub struct Theme {
    pub thermostat: MainScreenTheme,
    pub mode_select: ModeSelectTheme
}

impl Theme {
    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let toml_src = fs::read_to_string(file_path)?;
        let theme = toml::from_str(&toml_src)?;
        Ok(theme)
    }
}

impl Default for Theme {
    fn default() -> Self {
        let fonts = Fonts::new();

        // https://htmlcolorcodes.com/color-picker/
        // Pick dial colour, then use one level lighter for bg, one level higher for dot

        let heat_bg = theme_de::colour_from_hex("#F17E3B").unwrap();
        let heat_dial = theme_de::colour_from_hex("#E65D10").unwrap();
        let heat_dial_dot = theme_de::colour_from_hex("#C4500E").unwrap();

        let cool_bg = theme_de::colour_from_hex("#3B72F1").unwrap();
        let cool_dial = theme_de::colour_from_hex("#1050E6").unwrap();
        let cool_dial_dot = theme_de::colour_from_hex("#0E44C4").unwrap();

        Theme {
            thermostat: MainScreenTheme {
                fg_colour: Bgr888::WHITE,
                bg_colour: Bgr888::BLACK,
                bg_heat_colour: heat_bg,
                bg_cool_colour: cool_bg,

                gauge: GaugeStyle {
                    fg_colour: Bgr888::WHITE,
                    arc_dia: 260,
                    arc_width: 20,
                    arc_start_deg: 120.0,
                    arc_sweed_deg: 300.0,

                    target_font: fonts.font_def(FontName::Bold, 100),
                    target_decimal_font: fonts.font_def(FontName::Bold, 40),
                    current_font: fonts.font_def(FontName::Regular, 20),

                    arc_bg_colour: Bgr888::CSS_DIM_GRAY,

                    arc_heat_colour: heat_dial,
                    arc_heat_dot_colour: heat_dial_dot,

                    arc_cool_colour: cool_dial,
                    arc_cool_dot_colour: cool_dial_dot,

                    arc_target_dot_dia: 30,

                    arc_temp_dot_dia: 12,
                    arc_temp_dot_colour: Bgr888::CSS_SILVER,
                    arc_temp_text_dia: 220
                },

                away_icon_center: Point { x: 160, y: 230 },
                away_icon: IconStyle {
                    icon_font: fonts.font_def(FontName::Icon, 42),
                    icon: "\u{e50b}".to_string(),
                    colour: Bgr888::CSS_DIM_GRAY
                }
            },
            mode_select: ModeSelectTheme {
                bg_colour: Bgr888::BLACK,

                icon_heat_colour: heat_dial,
                icon_cool_colour: cool_dial,
                icon_center: Point { x: 160, y: 25 },

                mode_icon: IconStyle {
                    icon_font: fonts.font_def(FontName::Icon, 42),
                    icon: "\u{f72e}".to_string(),
                    colour: Bgr888::CSS_LIGHT_GRAY
                },

                mode_list: ListStyle {
                    colour: Bgr888::CSS_LIGHT_GRAY,
                    label_font: fonts.font_def(FontName::Bold, 36),

                    icon_font: fonts.font_def(FontName::Icon, 20),
                    selected_icon: "\u{f00c}".to_string(),

                    highlight_text_colour: Bgr888::WHITE,
                    highlight_rect: RectStyle {
                        stroke: None,
                        fill_colour: Some(Bgr888::CSS_DODGER_BLUE),
                        corner_radius: 18
                    },

                    row_size: Size::new(140, 40)
                }
            }
        }
    }
}

/// Main screen
///
/// Customize the look and feel of the main thermostat screen.
///
/// ```toml
/// [main_screen]
/// fg_color = "#00ff00"
/// ```
#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct MainScreenTheme {
    /// Colour of text on main screen, default "#ffffff"
    #[serde(deserialize_with = "theme_de::colour")]
    pub fg_colour: Bgr888,

    /// Background colour, default "#000000"
    #[serde(deserialize_with = "theme_de::colour")]
    pub bg_colour: Bgr888,

    /// Background colour when heating is turned on, default "#F17E3B"
    #[serde(deserialize_with = "theme_de::colour")]
    pub bg_heat_colour: Bgr888,

    /// Background colour when cooling is turned on, default "#3B72F1"
    #[serde(deserialize_with = "theme_de::colour")]
    pub bg_cool_colour: Bgr888,

    pub gauge: GaugeStyle,

    /// Position of away icon, default `[160, 230]`
    #[serde(deserialize_with = "theme_de::point")]
    pub away_icon_center: Point,

    /// Away icon styling, default `{ icon_font: "Icon:42", icon: "\u{e50b}", colour: "#696969" }`
    pub away_icon: IconStyle
}

impl Default for MainScreenTheme {
    fn default() -> Self {
        Theme::default().thermostat
    }
}

/// Mode select screen
///
/// Customize the look and feel of the mode select screen.
///
/// ```toml
/// [mode_select]
/// bg_color = "#000000"
/// ```
#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct ModeSelectTheme {
    /// Background colour, default "#000000"
    #[serde(deserialize_with = "theme_de::colour")]
    pub bg_colour: Bgr888,

    /// Heat mode icon colour, default "#E65D10"
    #[serde(deserialize_with = "theme_de::colour")]
    pub icon_heat_colour: Bgr888,

    /// Cool mode icon colour, default "#1050E6"
    #[serde(deserialize_with = "theme_de::colour")]
    pub icon_cool_colour: Bgr888,

    /// Position of mode icon, default `[160, 25]`
    #[serde(deserialize_with = "theme_de::point")]
    pub icon_center: Point,

    /// Mode icon styling, default `{ icon_font: "Icon:42", icon: "\u{f72e}", colour: "#696969" }`
    pub mode_icon: IconStyle,

    pub mode_list: ListStyle
}

impl Default for ModeSelectTheme {
    fn default() -> Self {
        Theme::default().mode_select
    }
}
