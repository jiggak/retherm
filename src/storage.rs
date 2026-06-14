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

use std::{fs, path::{Path, PathBuf}, sync::mpsc::{Sender, channel}, thread};

use anyhow::{Result, anyhow};
use log::{info, warn};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    config::Config,
    env,
    events::{Event, EventHandler},
    state::{HvacMode, ThermostatState}
};

pub struct Storage {
    backend: StorageBackend,
    write_thread: Sender<Storable>
}

impl Storage {
    pub fn new(config: &Config) -> Result<Self> {
        if !config.storage_dir.is_dir() {
            Err(anyhow!("Directory {:?} does not exist", config.storage_dir))
        } else {
            let backend = StorageBackend::new(config.storage_dir.clone());
            let write_thread = start_write_thread(backend.clone());
            Ok(Self {
                backend, write_thread
            })
        }
    }

    pub fn read_state(&self) -> Result<ThermostatState> {
        let state = if let Some(state) = self.backend.read(env::state_file_name())? {
            ThermostatState::from(&state)
        } else {
            warn!("State does not exist, using default");
            ThermostatState::default()
        };

        info!("Loaded state {:?}", state);

        Ok(state)
    }
}

fn start_write_thread(backend: StorageBackend) -> Sender<Storable> {
    let (tx, rx) = channel::<Storable>();

    thread::spawn(move || {
        while let Ok(data) = rx.recv() {
            match data {
                Storable::State(state) => {
                    let state = StoredState::from(&state);
                    backend.write(env::state_file_name(), state).unwrap();
                }
            }
        }
    });

    tx
}

impl EventHandler for Storage {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::State(state) = event {
            self.write_thread.send(Storable::State(state.clone()))?;
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, PartialEq)]
struct StoredState {
    target_temp: f32,
    current_temp: f32,
    mode: HvacMode,
}

impl From<&ThermostatState> for StoredState {
    fn from(value: &ThermostatState) -> Self {
        Self {
            target_temp: value.target_temp,
            current_temp: value.current_temp,
            mode: value.mode,
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

enum Storable {
    State(ThermostatState)
}

#[derive(Clone)]
struct StorageBackend {
    storage_dir: PathBuf
}

impl StorageBackend {
    fn new(storage_dir: PathBuf) -> Self {
        Self { storage_dir }
    }

    fn read<P, T>(&self, file_name: P) -> Result<Option<T>>
        where P: AsRef<Path>, T: DeserializeOwned
    {
        let file_path = self.storage_dir.join(file_name);
        info!("Loading file {:?}", file_path.file_name());

        let state = if file_path.is_file() {
            let toml_src = fs::read_to_string(&file_path)?;
            Some(toml::from_str(&toml_src)?)
        } else {
            None
        };

        Ok(state)
    }

    fn write<P, T>(&self, file_name: P, value: T) -> Result<()>
        where P: AsRef<Path>, T: Serialize + DeserializeOwned + PartialEq
    {
        let file_path = self.storage_dir.join(file_name.as_ref());

        // In an attempt to avoid excessive NAND writes,
        // serialize and write if data has changed.
        let toml_src = match self.read::<P, T>(file_name)? {
            None => Some(toml::to_string(&value)?),
            Some(existing) if existing != value => {
                Some(toml::to_string(&value)?)
            }
            _ => None
        };

        if let Some(toml_src) = toml_src {
            info!("Saving to file {:?}", file_path);

            fs::write(&file_path, toml_src)?;
        }

        Ok(())
    }
}
