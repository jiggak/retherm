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

use std::{fs, path::PathBuf, sync::mpsc::{Sender, channel}, thread};

use anyhow::{Result, anyhow};
use log::{info, warn};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    config::Config,
    events::{Event, EventHandler},
    state::{HvacMode, ThermostatState}
};

pub struct Storage {
    state_storage: StorageBackend,
    write_thread: Sender<ThermostatState>
}

impl Storage {
    pub fn new(config: &Config) -> Result<Self> {
        let state_dir = config.state_file_path.parent()
            .ok_or(anyhow!("Unable to get parent of state file path"))?;
        if !state_dir.is_dir() {
            Err(anyhow!("Directory {:?} does not exist", state_dir))
        } else {
            let state_storage = StorageBackend::new(config.state_file_path.clone());
            let write_thread = start_write_thread(state_storage.clone());
            Ok(Self {
                state_storage, write_thread
            })
        }
    }

    pub fn read_state(&self) -> Result<ThermostatState> {
        let state = if let Some(state) = self.state_storage.read()? {
            ThermostatState::from(&state)
        } else {
            warn!("State does not exist, using default");
            ThermostatState::default()
        };

        info!("Loaded state {:?}", state);

        Ok(state)
    }
}

fn start_write_thread(state_storage: StorageBackend) -> Sender<ThermostatState> {
    let (tx, rx) = channel::<ThermostatState>();

    thread::spawn(move || {
        while let Ok(state) = rx.recv() {
            let state = StoredState::from(&state);
            state_storage.write(state).unwrap();
        }
    });

    tx
}

impl EventHandler for Storage {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::State(state) = event {
            self.write_thread.send(state.clone())?;
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, PartialEq)]
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

#[derive(Clone)]
struct StorageBackend {
    file_path: PathBuf
}

impl StorageBackend {
    fn new(file_path: PathBuf) -> Self {
        Self { file_path }
    }

    fn read<T>(&self) -> Result<Option<T>>
        where T: DeserializeOwned
    {
        info!("Loading file {:?}", self.file_path);

        let state = if self.file_path.is_file() {
            let toml_src = fs::read_to_string(&self.file_path)?;
            Some(toml::from_str(&toml_src)?)
        } else {
            None
        };

        Ok(state)
    }

    fn write<T>(&self, value: T) -> Result<()>
        where T: Serialize + DeserializeOwned + PartialEq
    {
        if let Some(existing) = self.read::<T>()? {
            if existing != value {
                info!("Saving to file {:?}", self.file_path);

                let toml_src = toml::to_string(&value)?;
                fs::write(&self.file_path, toml_src)?;
            }
        }

        Ok(())
    }
}
