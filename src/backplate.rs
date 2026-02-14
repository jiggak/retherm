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

use crate::{events::{Event, EventHandler, EventSender}, state::{HvacAction, HvacMode, ThermostatState}};

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
    hvac_control: C,
    state: ThermostatState
}

impl<S: EventSender, C: HvacControl> Backplate<S, C> {
    pub fn new(event_sender: S, hvac_control: C) -> Result<Self> {
        Ok(Self {
            event_sender,
            hvac_control,
            state: ThermostatState::default()
        })
    }

    fn set_target_temp(&mut self, temp: f32) -> Result<bool> {
        let changed = if temp != self.state.target_temp {
            self.state.target_temp = temp;
            self.apply_hvac_action()?;
            true
        } else {
            false
        };

        Ok(changed)
    }

    fn set_current_temp(&mut self, temp: f32) -> Result<bool> {
        let changed = if temp != self.state.current_temp {
            self.state.current_temp = temp;
            self.apply_hvac_action()?;
            true
        } else {
            false
        };

        Ok(changed)
    }

    fn set_mode(&mut self, mode: HvacMode) -> Result<bool> {
        let changed = if mode != self.state.mode {
            self.state.mode = mode;
            self.apply_hvac_action()?;
            true
        } else {
            false
        };

        Ok(changed)
    }

    fn set_action(&mut self, action: HvacAction) -> Result<()> {
        if action != self.state.action {
            self.state.action = action;
            self.hvac_control.switch_hvac(&action)?;
        }

        Ok(())
    }

    fn apply_hvac_action(&mut self) -> Result<()> {
        match self.state.mode {
            HvacMode::Heat => {
                if self.state.current_temp < self.state.target_temp {
                    self.set_action(HvacAction::Heating)?;
                } else {
                    self.set_action(HvacAction::Idle)?;
                }
            }
            HvacMode::Cool => {
                if self.state.current_temp > self.state.target_temp {
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
            self.event_sender.send_event(Event::State(self.state.clone()))?;
        }

        Ok(())
    }
}
