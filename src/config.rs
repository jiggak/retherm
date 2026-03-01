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

use std::{fs, path::Path, time::Duration};

use anyhow::Result;
use serde::Deserialize;

mod config_de;
mod schedule_config;

pub use schedule_config::*;

use crate::state::HvacMode;

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub away_mode: AwayConfig,
    pub backplate: BackplateConfig,
    pub home_assistant: HomeAssistantConfig,
    pub backlight: BacklightConfig,
    pub schedule_heat: Vec<ScheduleConfig>,
    pub schedule_cool: Vec<ScheduleConfig>
}

impl Config {
    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let toml_src = fs::read_to_string(file_path)?;
        let config = toml::from_str(&toml_src)?;
        Ok(config)
    }

    pub fn schedule_for_mode(&self, mode: &HvacMode) -> Option<&[ScheduleConfig]> {
        match mode {
            HvacMode::Heat => {
                if self.schedule_heat.len() > 0 {
                    Some(&self.schedule_heat)
                } else {
                    None
                }
            }
            HvacMode::Cool => {
                if self.schedule_cool.len() > 0 {
                    Some(&self.schedule_cool)
                } else {
                    None
                }
            }
            _ => None
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            away_mode: AwayConfig::default(),
            backplate: BackplateConfig::default(),
            home_assistant: HomeAssistantConfig::default(),
            backlight: BacklightConfig::default(),
            schedule_heat: Vec::new(),
            schedule_cool: Vec::new()
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct BacklightConfig {
    pub brightness: u32,
    #[serde(deserialize_with = "config_de::duration")]
    pub timeout: Duration
}

impl Default for BacklightConfig {
    fn default() -> Self {
        Self {
            brightness: 108,
            timeout: Duration::from_secs(15)
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct AwayConfig {
    /// Away temp for heating mode
    pub temp_heat: f32,

    /// Away temp for cooling mode
    pub temp_cool: f32,

    /// Duration of no proximity movement before going into away mode
    #[serde(deserialize_with = "config_de::duration")]
    pub timeout: Duration
}

impl Default for AwayConfig {
    fn default() -> Self {
        Self {
            temp_heat: 16.0,
            temp_cool: 22.0,
            timeout: Duration::from_mins(30)
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct BackplateConfig {
    /// Minimum near proximity value to be consider as movement
    pub near_pir_threshold: u16,

    /// Path to backplate serial device file
    pub serial_port: String,

    /// HVAC wiring configuration
    pub wiring: WireConfig
}

impl Default for BackplateConfig {
    fn default() -> Self {
        Self {
            near_pir_threshold: 15,
            serial_port: String::from("/dev/ttyO2"),
            wiring: WireConfig::HeatAndCool {
                heat_wire: WireId::W1,
                cool_wire: WireId::Y1
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub enum WireId {
    W1, Y1, G, OB, W2, Y2, Star
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum WireConfig {
    HeatAndCool {
        heat_wire: WireId,
        cool_wire: WireId
    }
}
