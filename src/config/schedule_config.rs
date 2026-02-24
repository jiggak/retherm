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

use chrono::{NaiveTime, Weekday};
use serde::Deserialize;

use super::config_de;

#[derive(Deserialize, Debug, Clone)]
pub struct ScheduleConfig {
    pub days_of_week: DaysOfWeek,
    pub set_points: Vec<SetPoint>
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum DaysOfWeek {
    Range(WeekDayRange),
    List(Vec<WeekDay>)
}

impl DaysOfWeek {
    pub fn normalize(&self) -> Vec<Weekday> {
        match self {
            DaysOfWeek::List(days) => days.iter()
                .map(|d| d.to_chrono())
                .collect(),
            DaysOfWeek::Range(range) => {
                match range {
                    WeekDayRange::EveryDay => vec![
                        Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri, Weekday::Sat, Weekday::Sun
                    ],
                    WeekDayRange::WeekDays => vec![
                        Weekday::Mon, Weekday::Tue, Weekday::Wed, Weekday::Thu, Weekday::Fri
                    ],
                    WeekDayRange::WeekEnd => vec![
                        Weekday::Sat, Weekday::Sun
                    ]
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub enum WeekDayRange {
    EveryDay,
    WeekDays,
    WeekEnd
}

#[derive(Deserialize, Debug, Clone)]
pub enum WeekDay {
    Mon,
    Tue,
    Wed,
    Thur,
    Fri,
    Sat,
    Sun
}

impl WeekDay {
    pub fn to_chrono(&self) -> Weekday {
        match self {
            Self::Mon => Weekday::Mon,
            Self::Tue => Weekday::Tue,
            Self::Wed => Weekday::Wed,
            Self::Thur => Weekday::Thu,
            Self::Fri => Weekday::Fri,
            Self::Sat => Weekday::Sat,
            Self::Sun => Weekday::Sun
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SetPoint {
    #[serde(deserialize_with = "config_de::time_of_day")]
    pub time: NaiveTime,
    pub temp: f32
}
