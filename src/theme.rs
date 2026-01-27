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
use embedded_ttf::{FontTextStyle, FontTextStyleBuilder};
use rusttype::Font;
use serde::Deserialize;

use crate::theme::font::{FontName, Fonts};

mod font;
mod theme_de;

#[derive(Deserialize)]
#[serde(default)]
pub struct Theme {
    pub gauge: GaugeTheme,
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

        Theme {
            gauge: GaugeTheme {
                fg_colour: Bgr888::WHITE,
                bg_colour: Bgr888::BLACK,

                arc_dia: 280,
                arc_width: 12,
                arc_start_deg: 120.0,
                arc_sweed_deg: 300.0,

                target_font: fonts.font_def(FontName::Bold, 100),
                target_decimal_font: fonts.font_def(FontName::Bold, 40),
                current_font: fonts.font_def(FontName::Regular, 20),

                arc_bg_colour: Bgr888::CSS_DIM_GRAY,

                arc_heat_colour: Bgr888::CSS_PERU,
                arc_heat_dot_colour: Bgr888::CSS_DARK_ORANGE,

                arc_cool_colour: Bgr888::CSS_ROYAL_BLUE,
                arc_cool_dot_colour: Bgr888::CSS_DODGER_BLUE,

                arc_target_dot_dia: 20,

                arc_temp_dot_dia: 10,
                arc_temp_dot_colour: Bgr888::CSS_SILVER,
                arc_temp_text_dia: 248
            },
            mode_select: ModeSelectTheme {
                fg_colour: Bgr888::CSS_LIGHT_GRAY,
                bg_colour: Bgr888::BLACK,

                label_font: fonts.font_def(FontName::Bold, 36),
                icon_font: fonts.font_def(FontName::Icon, 20),

                row_size: Size::new(140, 40),
                checkmark: "\u{f00c}".to_string(),

                highlight_text_colour: Bgr888::WHITE,
                highlight_rect: RectTheme {
                    stroke_width: None,
                    stroke_colour: None,
                    fill_colour: Some(Bgr888::CSS_DODGER_BLUE),
                    corner_radius: Some(18)
                }
            }
        }
    }
}

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

pub trait FontStyle<C: PixelColor> {
    fn font_style(&self, font: &FontDef<'static>) -> FontTextStyle<C>;
}

#[derive(Deserialize, Clone)]
pub struct RectTheme {
    pub stroke_width: Option<u32>,
    #[serde(deserialize_with = "theme_de::optional_colour")]
    pub stroke_colour: Option<Bgr888>,
    #[serde(deserialize_with = "theme_de::optional_colour")]
    pub fill_colour: Option<Bgr888>,
    pub corner_radius: Option<u32>
}

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct GaugeTheme {
    #[serde(deserialize_with = "theme_de::colour")]
    pub fg_colour: Bgr888,
    #[serde(deserialize_with = "theme_de::colour")]
    pub bg_colour: Bgr888,

    /// Diameter of guage arch
    pub arc_dia: u32,
    /// Width of arc
    pub arc_width: u32,
    /// Arc start angle; 0 degrees at 3'oclock
    pub arc_start_deg: f32,
    pub arc_sweed_deg: f32,

    /// Target temp decimal digit font
    #[serde(deserialize_with = "theme_de::font")]
    pub target_font: FontDef<'static>,
    /// Target temp fraction digit font
    #[serde(deserialize_with = "theme_de::font")]
    pub target_decimal_font: FontDef<'static>,
    /// Current temp font
    #[serde(deserialize_with = "theme_de::font")]
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

impl Default for GaugeTheme {
    fn default() -> Self {
        Theme::default().gauge
    }
}

impl FontStyle<Bgr888> for GaugeTheme {
    fn font_style(&self, font: &FontDef<'static>) -> FontTextStyle<Bgr888> {
        font.font_style(self.fg_colour, self.bg_colour)
    }
}

#[derive(Deserialize, Clone)]
#[serde(default)]
pub struct ModeSelectTheme {
    #[serde(deserialize_with = "theme_de::colour")]
    pub fg_colour: Bgr888,
    #[serde(deserialize_with = "theme_de::colour")]
    pub bg_colour: Bgr888,

    #[serde(deserialize_with = "theme_de::font")]
    pub label_font: FontDef<'static>,
    #[serde(deserialize_with = "theme_de::font")]
    pub icon_font: FontDef<'static>,

    #[serde(deserialize_with = "theme_de::size")]
    pub row_size: Size,
    pub checkmark: String,

    #[serde(deserialize_with = "theme_de::colour")]
    pub highlight_text_colour: Bgr888,
    pub highlight_rect: RectTheme
}

impl Default for ModeSelectTheme {
    fn default() -> Self {
        Theme::default().mode_select
    }
}

impl FontStyle<Bgr888> for ModeSelectTheme {
    fn font_style(&self, font: &FontDef<'static>) -> FontTextStyle<Bgr888> {
        font.font_style(self.fg_colour, self.bg_colour)
    }
}
