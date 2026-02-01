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

use anyhow::{Result, anyhow};
use esphome_api::proto::{ClimateAction, ClimateFanMode, ClimateMode, ClimateStateResponse};

use crate::events::{Event, EventHandler, EventSender};

#[cfg(feature = "device")]
mod backplate_device;
#[cfg(feature = "simulate")]
mod backplate_simulated;

#[cfg(feature = "device")]
pub fn hvac_control<S>(event_sender: S) -> Result<impl HvacControl>
    where S: EventSender + Send + 'static
{
    use backplate_device::DeviceBackplateThread;

    let backplate_thread = DeviceBackplateThread::start("/dev/ttyO2", event_sender)?;
    Ok(backplate_thread)
}

#[cfg(feature = "simulate")]
pub fn hvac_control<S>(_event_sender: S) -> Result<impl HvacControl>
    where S: EventSender + Send + 'static
{
    use backplate_simulated::SimulatedBackplate;

    Ok(SimulatedBackplate::new())
}

pub trait HvacControl {
    fn switch_hvac(&self, action: &HvacAction) -> Result<()>;
}

pub struct Backplate<S, C> {
    event_sender: S,
    hvac_state: HvacState,
    hvac_control: C
}

impl<S: EventSender, C: HvacControl> Backplate<S, C> {
    pub fn new(event_sender: S, hvac_control: C) -> Result<Self> {
        Ok(Self {
            event_sender,
            hvac_state: HvacState::default(),
            hvac_control
        })
    }

    fn set_target_temp(&mut self, temp: f32) -> Result<bool> {
        let changed = if temp != self.hvac_state.target_temp {
            self.hvac_state.target_temp = temp;
            self.apply_hvac_action()?;
            true
        } else {
            false
        };

        Ok(changed)
    }

    fn set_current_temp(&mut self, temp: f32) -> Result<bool> {
        let changed = if temp != self.hvac_state.current_temp {
            self.hvac_state.current_temp = temp;
            self.apply_hvac_action()?;
            true
        } else {
            false
        };

        Ok(changed)
    }

    fn set_mode(&mut self, mode: HvacMode) -> Result<bool> {
        let changed = if mode != self.hvac_state.mode {
            self.hvac_state.mode = mode;
            self.apply_hvac_action()?;
            true
        } else {
            false
        };

        Ok(changed)
    }

    fn set_action(&mut self, action: HvacAction) -> Result<()> {
        if action != self.hvac_state.action {
            self.hvac_state.action = action;
            self.hvac_control.switch_hvac(&action)?;
        }

        Ok(())
    }

    fn apply_hvac_action(&mut self) -> Result<()> {
        match self.hvac_state.mode {
            HvacMode::Heat => {
                if self.hvac_state.current_temp < self.hvac_state.target_temp {
                    self.set_action(HvacAction::Heating)?;
                } else {
                    self.set_action(HvacAction::Idle)?;
                }
            }
            HvacMode::Cool => {
                if self.hvac_state.current_temp > self.hvac_state.target_temp {
                    self.set_action(HvacAction::Cooling)?;
                } else {
                    self.set_action(HvacAction::Idle)?;
                }
            }
            HvacMode::Off => {
                self.set_action(HvacAction::Idle)?;
            }
            _ => { }
        }

        Ok(())
    }
}

impl<S: EventSender, C: HvacControl> EventHandler for Backplate<S, C> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        let send_state_event = match event {
            Event::SetMode(mode) => {
                self.set_mode(*mode)?
            }
            Event::SetTargetTemp(temp) => {
                self.set_target_temp(*temp)?
            }
            Event::SetCurrentTemp(temp) => {
                self.set_current_temp(*temp)?
            }
            _ => false
        };

        if send_state_event {
            self.event_sender.send_event(Event::HvacState(self.hvac_state.clone()))?;
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

impl HvacState {
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
            HvacAction::Cooling => Self::Cooling,
        }
    }
}
