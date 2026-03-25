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

use crate::{env, state::HvacMode};

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    pub away_mode: AwayConfig,
    pub backplate: BackplateConfig,
    pub home_assistant: HomeAssistantConfig,
    pub backlight: BacklightConfig,
    pub schedule_heat: Vec<ScheduleConfig>,
    pub schedule_cool: Vec<ScheduleConfig>,
    pub temp_differential: f32
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
            schedule_cool: Vec::new(),
            temp_differential: 0.2
        }
    }
}

/// Home Assistant
///
/// ```toml
/// [home_assistant]
/// friendly_name = "Hallway"
/// encryption_key = "..."
/// ```
#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct HomeAssistantConfig {
    /// Object ID used internall by home assistant.
    /// Defaults to "climage.{node_name}".
    pub object_id: Option<String>,

    /// Listen address for ESP Home API server, default "0.0.0.0:6053"
    pub listen_addr: String,

    /// Encryption key as 32 byte base64 string. When not provided, the
    /// connection uses plaintext messages.
    /// See [ESP Home Native API](https://esphome.io/components/api/)
    /// for a tool that generates a random key.
    pub encryption_key: Option<String>,

    /// Server info (not typically displayed in Home Assistant).
    /// Defaults to "ReTherm {version}".
    pub server_info: String,

    /// Node name, defaults to the system hostname
    pub node_name: Option<String>,

    /// Friendly name displayed in as label for thermostat control
    pub friendly_name: String,

    /// Manufactuer name, defaults to "Nest"
    pub manufacturer: String,

    /// Model name, defaults to "Gen2 Thermostat"
    pub model: String,

    /// Mac address, defaults to address of system interface address
    pub mac_address: Option<String>
}

impl HomeAssistantConfig {
    pub fn get_object_id(&self) -> String {
        if let Some(object_id) = &self.object_id {
            object_id.clone()
        } else {
            format!("climate.{}", self.get_node_name())
        }
    }

    pub fn get_node_name(&self) -> String {
        let pkg_name = env::get_pkg_name();

        if let Some(node_name) = &self.node_name {
            node_name.clone()
        } else {
            match env::get_hostname() {
                Ok(hostname) => hostname,
                Err(e) => {
                    log::error!("get_hostname: '{e}'; using '{pkg_name}'");
                    pkg_name.into()
                }
            }
        }
    }

    pub fn get_mac_address(&self) -> String {
        const FAKE_MAC: &str = "01:02:03:04:05:06";

        if let Some(mac_addr) = &self.mac_address {
            mac_addr.clone()
        } else {
            match env::get_mac_addr() {
                Ok(mac_addr) => {
                    if let Some(mac_addr) = mac_addr {
                        mac_addr
                    } else {
                        log::warn!("get_mac_addr None; using '{FAKE_MAC}'");
                        FAKE_MAC.into()
                    }
                }
                Err(e) => {
                    log::error!("get_mac_addr: '{e}'; using '{FAKE_MAC}'");
                    FAKE_MAC.into()
                }
            }
        }
    }
}

impl Default for HomeAssistantConfig {
    fn default() -> Self {
        Self {
            object_id: None,
            listen_addr: "0.0.0.0:6053".to_string(),
            encryption_key: None,
            server_info: format!("ReTherm {}", env::get_pkg_ver()),
            node_name: None,
            friendly_name: "ReTherm Thermostat".to_string(),
            manufacturer: "Nest".to_string(),
            model: "Gen2 Thermostat".to_string(),
            mac_address: None
        }
    }
}

/// Backlight
///
/// ```toml
/// [backlight]
/// brightness = 108
/// timeout = "15s"
/// ```
#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct BacklightConfig {
    /// Screen brightness, defaults to 108 (max 120)
    pub brightness: u32,

    /// Timeout before screen turns off, defaults to "15s"
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


/// Away Mode
///
/// ```toml
/// [away_mode]
/// temp_heat = 16.0
/// temp_cool = 20.0
/// timeout = "0s"
/// ```
#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct AwayConfig {
    /// Away temp for heating mode, default 16.0
    pub temp_heat: f32,

    /// Away temp for cooling mode, default 22.0
    pub temp_cool: f32,

    /// Duration of no proximity movement before going into away mode,
    /// or set to zero to disable away mode. Default "30m".
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

/// Backplate
///
/// ```toml
/// [backplate]
/// near_pir_threshold = 15
/// serial_port = "/dev/ttyO2"
/// wiring = { heat_wire: "W1", cool_wire: "Y1" }
/// ```
#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct BackplateConfig {
    /// Minimum near proximity value to be consider as movement, default 15
    pub near_pir_threshold: u16,

    /// Path to backplate serial device file, default "/dev/ttyO2"
    pub serial_port: String,

    /// HVAC wiring configuration, default `{ heat_wire: "W1", cool_wire: "Y1" }`.
    /// Valid wire names: W1, Y1, G, OB, W2, Y2, Star.
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
