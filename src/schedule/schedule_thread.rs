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
    thread,
    sync::mpsc::{RecvTimeoutError, Sender, channel},
    time::Duration
};

use anyhow::Result;
use chrono::Local;

use crate::events::{Event, EventSender};
use super::schedule_model::Schedule;

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
