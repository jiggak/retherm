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

use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};
use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config,
    events::{Event, EventHandler},
    state::{HvacMode, ThermostatState}
};

pub struct Storage {
    state_file_path: PathBuf
}

impl Storage {
    pub fn new(config: &Config) -> Result<Self> {
        let state_dir = config.state_file_path.parent()
            .ok_or(anyhow!("Unable to get parent of state file path"))?;
        if !state_dir.is_dir() {
            Err(anyhow!("Directory {:?} does not exist", state_dir))
        } else {
            Ok(Self {
                state_file_path: config.state_file_path.clone()
            })
        }
    }

    pub fn read_state(&self) -> Result<ThermostatState> {
        info!("Loading state from {:?}", self.state_file_path);
        let state = if self.state_file_path.is_file() {
            let toml_src = fs::read_to_string(&self.state_file_path)?;
            let state_storage:StoredState = toml::from_str(&toml_src)?;
            ThermostatState::from(&state_storage)
        } else {
            warn!("State does not exist, using default");
            ThermostatState::default()
        };

        info!("Loaded state {:?}", state);

        Ok(state)
    }

    fn save_state(&self, state: &ThermostatState) -> Result<()> {
        info!("Saving state to {:?}", self.state_file_path);
        let state_storage = StoredState::from(state);
        let toml_src = toml::to_string(&state_storage)?;
        fs::write(&self.state_file_path, toml_src)?;
        Ok(())
    }
}

impl EventHandler for Storage {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::State(state) = event {
            self.save_state(&state)?;
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
struct StoredState {
    target_temp: f32,
    current_temp: f32,
    mode: HvacMode
}

impl From<&ThermostatState> for StoredState {
    fn from(value: &ThermostatState) -> Self {
        Self {
            target_temp: value.target_temp,
            current_temp: value.current_temp,
            mode: value.mode
        }
    }
}

impl From<&StoredState> for ThermostatState {
    fn from(value: &StoredState) -> Self {
        Self {
            target_temp: value.target_temp,
            current_temp: value.current_temp,
            mode: value.mode,
            ..Default::default()
        }
    }
}
