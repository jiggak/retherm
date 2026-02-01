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

use serde::{Deserialize, de::{self, Visitor}};

use super::{font_def::FontDef, fonts::{FontName, Fonts}};

struct FontDefVisitor {
    fonts: Fonts
}

impl<'de> Visitor<'de> for FontDefVisitor {
    type Value = FontDef<'static>;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("string in the format <font>:<font_size>")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        let (name, size) = v.split_once(":")
            .ok_or(de::Error::custom("Missing `:` in font def string"))?;

        let name: FontName = name.parse()
            .map_err(|e| de::Error::custom(e))?;

        let size: u32 = size.parse()
            .map_err(|_| de::Error::custom(format!("Invalid font size `{}`", size)))?;

        Ok(self.fonts.font_def(name, size))
    }
}

impl<'de> Deserialize<'de> for FontDef<'static> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: serde::Deserializer<'de>
    {
        // TODO can I somehow have a single instance of `Fonts`?
        let visitor = FontDefVisitor { fonts: Fonts::new() };
        deserializer.deserialize_any(visitor)
    }
}
