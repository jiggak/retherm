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

use std::{fs, path::Path};

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct Config {
    pub home_assistant: HomeAssistantConfig
}

impl Config {
    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let toml_src = fs::read_to_string(file_path)?;
        let config = toml::from_str(&toml_src)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            home_assistant: HomeAssistantConfig::default()
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(default)]
pub struct HomeAssistantConfig {
    pub listen_addr: String,
    pub encryption_key: Option<String>,
    pub server_info: String,
    pub node_name: String,
    pub friendly_name: String,
    pub manufacturer: String,
    pub model: String
}

impl Default for HomeAssistantConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0:6053".to_string(),
            encryption_key: None,
            server_info: format!("ReTherm {}", env!("CARGO_PKG_VERSION")),
            node_name: "retherm-thermostat".to_string(),
            friendly_name: "ReTherm Thermostat".to_string(),
            manufacturer: "Nest".to_string(),
            model: "Gen2 Thermostat".to_string()
        }
    }
}
