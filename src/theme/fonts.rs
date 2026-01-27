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

use rusttype::Font;

use crate::theme::FontDef;

pub struct Fonts {
    regular: Font<'static>,
    bold: Font<'static>,
    icon: Font<'static>
}

impl Fonts {
    pub fn new() -> Self {
        let roboto_reg = include_bytes!("../../assets/roboto/Roboto-Regular.ttf");
        let roboto_bold = include_bytes!("../../assets/roboto/Roboto-Bold.ttf");
        let fa_solid = include_bytes!("../../assets/fontawesome-free-7.1.0/Font Awesome 7 Free-Solid-900.otf");

        let regular = Font::try_from_bytes(roboto_reg).expect("valid font");
        let bold = Font::try_from_bytes(roboto_bold).expect("valid font");
        let icon = Font::try_from_bytes(fa_solid).expect("valid font");

        Self { regular, bold, icon }
    }

    pub fn font_def(&self, name: FontName, size: u32) -> FontDef<'static> {
        let font = match name {
            FontName::Regular => &self.regular,
            FontName::Bold => &self.bold,
            FontName::Icon => &self.icon
        };

        FontDef::new(font, size)
    }
}

pub enum FontName {
    Regular,
    Bold,
    Icon
}

impl std::str::FromStr for FontName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "regular" => Ok(Self::Regular),
            "bold" => Ok(Self::Bold),
            "icon" => Ok(Self::Icon),
            s => Err(format!("Unsupported font name `{}`", s))
        }
    }
}
