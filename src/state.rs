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

use esphome_api::proto::{
    ClimateAction, ClimateFanMode, ClimateMode, ClimateStateResponse
};

#[derive(Debug, Clone)]
pub struct ThermostatState {
    pub target_temp: f32,
    pub current_temp: f32,
    pub mode: HvacMode,
    pub action: HvacAction,
    pub away: bool
}

impl ThermostatState {
    pub const MIN_TEMP: f32 = 9.0;
    pub const MAX_TEMP: f32 = 32.0;

    /// Attempt to set target temp and return `true` if successful.
    /// Return `false` if value is outside of min/max range, or if value
    /// equals current target temp.
    pub fn set_target_temp(&mut self, val: f32) -> bool {
        if val > Self::MIN_TEMP && val < Self::MAX_TEMP && val != self.target_temp {
            self.target_temp = val;
            true
        } else {
            false
        }
    }
}

impl Default for ThermostatState {
    fn default() -> Self {
        Self {
            target_temp: 19.5,
            current_temp: 20.0,
            action: HvacAction::Idle,
            mode: HvacMode::Heat,
            away: false
        }
    }
}

impl From<ThermostatState> for ClimateStateResponse {
    fn from(value: ThermostatState) -> Self {
        let mut state = Self::default();
        state.set_fan_mode(ClimateFanMode::ClimateFanAuto);

        state.set_action(value.action.into());
        state.set_mode(value.mode.into());
        state.current_temperature = value.current_temp;
        state.target_temperature = value.target_temp;

        state
    }
}

impl From<&ThermostatState> for ClimateStateResponse {
    fn from(value: &ThermostatState) -> Self {
        let mut state = Self::default();
        state.set_fan_mode(ClimateFanMode::ClimateFanAuto);

        state.set_action(value.action.into());
        state.set_mode(value.mode.into());
        state.current_temperature = value.current_temp;
        state.target_temperature = value.target_temp;

        state
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HvacMode {
    Off,
    Auto,
    Heat,
    Cool
}

impl TryFrom<ClimateMode> for HvacMode {
    type Error = anyhow::Error;

    fn try_from(value: ClimateMode) -> anyhow::Result<Self> {
        Ok(match value {
            ClimateMode::Off => Self::Off,
            ClimateMode::Auto => Self::Auto,
            ClimateMode::Heat => Self::Heat,
            ClimateMode::Cool => Self::Cool,
            v => return Err(anyhow::anyhow!("Unsupported climate mode {v:?}"))
        })
    }
}

impl From<HvacMode> for ClimateMode {
    fn from(value: HvacMode) -> Self {
        match value {
            HvacMode::Off => Self::Off,
            HvacMode::Auto => Self::Auto,
            HvacMode::Heat => Self::Heat,
            HvacMode::Cool => Self::Cool
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HvacAction {
    Idle,
    Heating,
    Cooling
}

impl From<HvacAction> for ClimateAction {
    fn from(value: HvacAction) -> Self {
        match value {
            HvacAction::Idle => Self::Idle,
            HvacAction::Heating => Self::Heating,
            HvacAction::Cooling => Self::Cooling
        }
    }
}
