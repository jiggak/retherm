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

use std::{
    collections::HashMap,
    sync::mpsc::{RecvTimeoutError, Sender, channel},
    thread,
    time::Duration
};

use anyhow::Result;
use chrono::{DateTime, Datelike, Local, NaiveTime, Weekday};
use log::info;

use crate::{config::ScheduleConfig, events::{Event, EventSender}};

type ScheduleMap = HashMap<Weekday, HashMap<NaiveTime, f32>>;

pub struct ScheduleThread {
    sender: Sender<()>
}

impl ScheduleThread {
    pub fn start<S>(mut schedule: Schedule, event_sender: S) -> Self
        where S: EventSender + Send + 'static
    {
        let tick_delay = Duration::from_secs(1);

        let (sender, receiver) = channel();

        thread::spawn(move || {
            loop {
                if let Some(temp) = schedule.get_target_temp(Local::now()) {
                    event_sender.send_event(Event::SetTargetTemp(temp))
                        .expect("Schedule event sender should send");
                }

                match receiver.recv_timeout(tick_delay) {
                    Err(RecvTimeoutError::Timeout) => continue,
                    _ => break
                }
            }
        });

        Self { sender }
    }

    pub fn stop(self) -> Result<()> {
        Ok(self.sender.send(())?)
    }
}

fn week_schedule(schedule: &[ScheduleConfig]) -> ScheduleMap {
    let mut week_schedule = HashMap::new();

    for s in schedule {
        for day in s.days_of_week.normalize() {
            if !week_schedule.contains_key(&day) {
                week_schedule.insert(day, HashMap::new());
            }

            let day_schedle = week_schedule.get_mut(&day).unwrap();
            for p in &s.set_points {
                day_schedle.entry(p.time)
                    .and_modify(|temp| *temp = p.temp)
                    .insert_entry(p.temp);
            }
        }
    }

    week_schedule
}

#[derive(Debug)]
pub struct Schedule {
    schedule: ScheduleMap,
    max_age: Duration,
    last_set_point: Option<f32>
}

impl Schedule {
    pub fn new(schedule: &[ScheduleConfig]) -> Self {
        let schedule = week_schedule(schedule);
        Self {
            schedule,
            max_age: Duration::from_secs(2),
            last_set_point: None
        }
    }

    pub fn get_target_temp(&mut self, now: DateTime<Local>) -> Option<f32> {
        let weekday = now.weekday();
        let time_of_day = now.time();

        if let Some(set_points) = self.schedule.get(&weekday) {
            for (set_point_time, set_point_temp) in set_points {
                // test if set point has been reached, or
                if time_of_day >= *set_point_time
                    // consider set point reached if time is within small range
                    // this is to account for (unlikely) unreliable thread delay
                    && time_of_day <= *set_point_time + self.max_age
                    // don't repreat reporting setpoint more than once
                    && self.last_set_point.is_none()
                {
                    info!("Set point reached {set_point_time} {set_point_temp}");
                    self.last_set_point = Some(*set_point_temp);
                    return Some(*set_point_temp);
                }
            }
        }

        self.last_set_point = None;
        None
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, prelude::*};

    use crate::config::{DaysOfWeek, ScheduleConfig, SetPoint, WeekDayRange};
    use super::Schedule;

    fn daily_morning_temp_increase() -> Schedule {
        Schedule::new(&[
            ScheduleConfig {
                days_of_week: DaysOfWeek::Range(WeekDayRange::EveryDay),
                set_points: vec![
                    SetPoint {
                        time: NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                        temp: 20.0
                    },
                    SetPoint {
                        time: NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                        temp: 16.0
                    }
                ]
            }
        ])
    }

    fn tick(date: DateTime<Local>) -> DateTime<Local> {
        date + Duration::seconds(1)
    }

    #[test]
    fn basic_schedule() {
        let mut schedule = daily_morning_temp_increase();

        let mut date = Local.with_ymd_and_hms(2026, 2, 23, 8, 0, 0).unwrap();

        assert_eq!(schedule.get_target_temp(date), Some(20.0));

        date = tick(date);

        assert_eq!(schedule.get_target_temp(date), None);

        let date = Local.with_ymd_and_hms(2026, 2, 23, 10, 0, 0).unwrap();
        assert_eq!(schedule.get_target_temp(date), Some(16.0));
    }

    #[test]
    fn resileant_clock_skip() {
        let mut schedule = daily_morning_temp_increase();

        let mut date = Local.with_ymd_and_hms(2026, 2, 23, 7, 59, 59).unwrap();

        assert_eq!(schedule.get_target_temp(date), None);

        // clock reaches nesxt set point
        date = tick(date);

        // next tick advances one sec past set point
        date = tick(date);

        assert_eq!(schedule.get_target_temp(date), Some(20.0));
    }
}
