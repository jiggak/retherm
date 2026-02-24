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
use chrono::{DateTime, Datelike, Local, NaiveTime, SubsecRound, Weekday};

use crate::{config::ScheduleConfig, events::{Event, EventSender}};

type ScheduleMap = HashMap<Weekday, HashMap<NaiveTime, f32>>;

pub struct ScheduleThread {
    sender: Sender<()>
}

impl ScheduleThread {
    pub fn start<S>(schedule: Schedule, event_sender: S) -> Self
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

pub struct Schedule {
    schedule: ScheduleMap
}

impl Schedule {
    pub fn new(schedule: &[ScheduleConfig]) -> Self {
        let schedule = week_schedule(schedule);
        Self {
            schedule
        }
    }

    pub fn get_target_temp(&self, now: DateTime<Local>) -> Option<f32> {
        let now = now.trunc_subsecs(0);
        let weekday = now.weekday();
        let time_of_day = now.time();

        if let Some(set_points) = self.schedule.get(&weekday) {
            for (time, temp) in set_points {
                // FIXME I need to make this more resilient to account for "tick"
                // possibly not occuring when expected. There must be cases
                // where a thread ticking every second could become paused
                // or delayed in some way.
                if time_of_day == *time {
                    return Some(*temp);
                }
            }
        }

        None
    }
}
