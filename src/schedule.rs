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

use anyhow::Result;
use log::info;

use crate::{
    config::Config,
    events::{Event, EventHandler, EventSender},
    state::HvacMode
};

mod schedule_model;
mod schedule_thread;

use schedule_model::Schedule;
use schedule_thread::ScheduleThread;

pub struct ScheduleManager<S> {
    event_sender: S,
    schedule_thread: Option<ScheduleThread>,
    config: Config
}

impl<S: EventSender + Clone + Send + 'static> ScheduleManager<S> {
    pub fn new(config: &Config, event_sender: S) -> Self {
        Self {
            event_sender,
            schedule_thread: None,
            config: config.clone()
        }
    }

    pub fn start_schedule(&mut self, mode: &HvacMode) {
        if let Some(thread) = self.schedule_thread.take() {
            info!("Stop schedule clock thread");
            thread.stop()
                .expect("Schedule thread should stop");
        }

        let schedule = match mode {
            HvacMode::Heat => {
                if self.config.schedule_heat.len() > 0 {
                    Some(Schedule::new(&self.config.schedule_heat))
                } else {
                    info!("Empty heat schedule, skip clock thread");
                    None
                }
            }
            HvacMode::Cool => {
                if self.config.schedule_cool.len() > 0 {
                    Some(Schedule::new(&self.config.schedule_cool))
                } else {
                    info!("Empty cool schedule, skip clock thread");
                    None
                }
            }
            _ => None
        };

        self.schedule_thread = match schedule {
            Some(schedule) => {
                info!("Start schedule clock thread {:?}", schedule);
                Some(ScheduleThread::start(schedule, self.event_sender.clone()))
            }
            None => None
        };
    }
}

impl<S: EventSender + Clone + Send + 'static> EventHandler for ScheduleManager<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::SetMode(mode) = event {
            self.start_schedule(mode);
        }
        Ok(())
    }
}
