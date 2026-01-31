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

use embedded_graphics::{pixelcolor::Bgr888, prelude::{Point, Size}};
use serde::{Deserializer, de::{self, SeqAccess, Visitor}};

use crate::theme::{FontDef, fonts::{FontName, Fonts}};

pub fn colour<'de, D>(deserializer: D) -> Result<Bgr888, D::Error>
    where D: Deserializer<'de>
{
    struct ColourVisitor;

    impl<'de> Visitor<'de> for ColourVisitor {
        type Value = Bgr888;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a hex color string or [r, g, b]")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            let v = v.strip_prefix('#').unwrap_or(v);

            let val = u32::from_str_radix(v, 16)
                .map_err(E::custom)?;

            Ok(Bgr888::new(
                ((val >> 16) & 0xff) as u8,
                ((val >> 8) & 0xff) as u8,
                (val & 0xff) as u8
            ))
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: SeqAccess<'de>
        {
            let r: u8 = seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let g: u8 = seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &self))?;
            let b: u8 = seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(2, &self))?;

            Ok(Bgr888::new(r, g, b))
        }
    }

    deserializer.deserialize_any(ColourVisitor)
}

pub fn optional_colour<'de, D>(deserializer: D) -> Result<Option<Bgr888>, D::Error>
    where D: Deserializer<'de>
{
    struct OptionalColourVisitor;

    impl<'de> Visitor<'de> for OptionalColourVisitor {
        type Value = Option<Bgr888>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("null, a hex color string, or [r, g, b]")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> where E: de::Error {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> where E: de::Error {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where D: Deserializer<'de>
        {
            colour(deserializer).map(Some)
        }
    }

    deserializer.deserialize_option(OptionalColourVisitor)
}

pub fn size<'de, D>(deserializer: D) -> Result<Size, D::Error>
    where D: Deserializer<'de>
{
    struct SizeVisitor;

    impl<'de> Visitor<'de> for SizeVisitor {
        type Value = Size;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("[width, height]")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: SeqAccess<'de>
        {
            let width: u32 = seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let height: u32 = seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &self))?;

            Ok(Size::new(width, height))
        }
    }

    deserializer.deserialize_any(SizeVisitor)
}

pub fn point<'de, D>(deserializer: D) -> Result<Point, D::Error>
    where D: Deserializer<'de>
{
    struct PointVisitor;

    impl<'de> Visitor<'de> for PointVisitor {
        type Value = Point;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("[x, y]")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: SeqAccess<'de>
        {
            let x: i32 = seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;
            let y: i32 = seq.next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &self))?;

            Ok(Point::new(x, y))
        }
    }

    deserializer.deserialize_any(PointVisitor)
}

pub fn font<'de, D>(deserializer: D) -> Result<FontDef<'static>, D::Error>
    where D: Deserializer<'de>
{
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

    // TODO can I somehow have a single instance of `Fonts`?
    deserializer.deserialize_any(
        FontDefVisitor { fonts: Fonts::new() }
    )
}
