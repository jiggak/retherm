/*
 * Nest UI - Home Assistant native thermostat interface
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

use anyhow::{Result, anyhow};
use esphome_api::proto::{ClimateAction, ClimateFanMode, ClimateMode, ClimateStateResponse};

use crate::events::{Event, EventHandler, EventOrigin, EventSender};

pub struct Backplate<S> {
    event_sender: S,
    hvac_state: HvacState
}

impl<S: EventSender> Backplate<S> {
    pub fn new(event_sender: S) -> Self {
        Self {
            event_sender,
            hvac_state: HvacState::default()
        }
    }
}

impl<S: EventSender> EventHandler for Backplate<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::Hvac { state, origin } = event && origin != &EventOrigin::Backplate {
            self.hvac_state = state.clone();
            self.event_sender.send_event(Event::Hvac {
                state: self.hvac_state.clone(),
                origin: EventOrigin::Backplate
            })?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct HvacState {
    pub target_temp: f32,
    pub current_temp: f32,
    pub mode: HvacMode,
    pub action: HvacAction
}

impl Default for HvacState {
    fn default() -> Self {
        Self {
            target_temp: 19.5,
            current_temp: 20.0,
            action: HvacAction::Idle,
            mode: HvacMode::Heat
        }
    }
}

impl From<HvacState> for ClimateStateResponse {
    fn from(value: HvacState) -> Self {
        let mut state = Self::default();
        state.set_fan_mode(ClimateFanMode::ClimateFanAuto);

        state.set_action(value.action.into());
        state.set_mode(value.mode.into());
        state.current_temperature = value.current_temp;
        state.target_temperature = value.target_temp;

        state
    }
}

impl From<&HvacState> for ClimateStateResponse {
    fn from(value: &HvacState) -> Self {
        let mut state = Self::default();
        state.set_fan_mode(ClimateFanMode::ClimateFanAuto);

        state.set_action(value.action.clone().into());
        state.set_mode(value.mode.clone().into());
        state.current_temperature = value.current_temp;
        state.target_temperature = value.target_temp;

        state
    }
}

#[derive(Debug, Clone)]
pub enum HvacMode {
    Off,
    Auto,
    Heat,
    Cool
}

impl TryFrom<ClimateMode> for HvacMode {
    type Error = anyhow::Error;
    fn try_from(value: ClimateMode) -> Result<Self> {
        Ok(match value {
            ClimateMode::Off => Self::Off,
            ClimateMode::Auto => Self::Auto,
            ClimateMode::Heat => Self::Heat,
            ClimateMode::Cool => Self::Cool,
            _ => return Err(anyhow!(""))
        })
    }
}

impl From<HvacMode> for ClimateMode {
    fn from(value: HvacMode) -> Self {
        match value {
            HvacMode::Off => Self::Off,
            HvacMode::Auto => Self::Auto,
            HvacMode::Heat => Self::Heat,
            HvacMode::Cool => Self::Cool,
        }
    }
}

#[derive(Debug, Clone)]
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
            HvacAction::Cooling => Self::Cooling,
        }
    }
}
