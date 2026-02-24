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

use std::time::Duration;

use chrono::NaiveTime;
use serde::{Deserializer, de::{self, Visitor}};

pub fn duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where D: Deserializer<'de>
{
    struct DurationVisitor;

    impl<'de> Visitor<'de> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("duration as number of seconds, or string with time unit suffix [s,m,h]")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            let val = if let Some(v) = v.strip_suffix("s") {
                let secs = u64::from_str_radix(v, 10)
                    .map_err(E::custom)?;
                Duration::from_secs(secs)
            } else if let Some(v) = v.strip_suffix("m") {
                let mins = u64::from_str_radix(v, 10)
                    .map_err(E::custom)?;
                Duration::from_mins(mins)
            } else if let Some(v) = v.strip_suffix("m") {
                let hours = u64::from_str_radix(v, 10)
                    .map_err(E::custom)?;
                Duration::from_hours(hours)
            } else {
                return Err(E::custom("Duration suffix must be one of [s,m,h]"));
            };

            Ok(val)
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where E: de::Error
        {
            Ok(Duration::from_secs(v))
        }
    }

    deserializer.deserialize_any(DurationVisitor)
}

pub fn time_of_day<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
    where D: Deserializer<'de>
{
    struct TimeOfDayVisitor;

    impl<'de> Visitor<'de> for TimeOfDayVisitor {
        type Value = NaiveTime;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("time of day as hh:mm")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            let (hour, min) = v.split_once(':')
                .ok_or(E::custom("missing ':' delimeter"))?;
            let hour:u32 = hour.parse().map_err(E::custom)?;
            let min:u32 = min.parse().map_err(E::custom)?;

            let val = NaiveTime::from_hms_opt(hour, min, 0)
                .ok_or(E::custom(""))?;

            Ok(val)
        }
    }

    deserializer.deserialize_any(TimeOfDayVisitor)
}
