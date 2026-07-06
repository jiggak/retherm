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

use std::time::{Duration, Instant};

use anyhow::Result;
use esphome_api::proto::{
    ClimateAction, ClimateFanMode, ClimateMode, ClimatePreset, ClimateStateResponse
};
use serde::{Deserialize, Serialize};

use crate::{
    config::Config, events::{Event, EventHandler, EventSender}, timer::TimerId
};

#[derive(Debug, Clone)]
pub struct ThermostatState {
    pub target_temp: f32,
    pub current_temp: f32,
    pub mode: HvacMode,
    pub action: HvacAction,
    pub away: bool,
    pub lockout: Option<Duration>,
    /// Backplate connected flag
    pub backplate: bool,
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

    fn to_ha_state(&self) -> ClimateStateResponse {
        let mut state = ClimateStateResponse::default();
        state.set_fan_mode(ClimateFanMode::ClimateFanAuto);

        state.set_action(self.action.into());
        state.set_mode(self.mode.into());
        state.current_temperature = self.current_temp;
        state.target_temperature = self.target_temp;
        state.preset = if self.away {
            ClimatePreset::Away as i32
        } else {
            ClimatePreset::None as i32
        };

        state
    }
}

impl Default for ThermostatState {
    fn default() -> Self {
        Self {
            target_temp: 19.5,
            current_temp: 20.0,
            action: HvacAction::Idle,
            mode: HvacMode::Heat,
            away: false,
            lockout: None,
            backplate: false,
        }
    }
}

impl From<ThermostatState> for ClimateStateResponse {
    fn from(value: ThermostatState) -> Self {
        value.to_ha_state()
    }
}

impl From<&ThermostatState> for ClimateStateResponse {
    fn from(value: &ThermostatState) -> Self {
        value.to_ha_state()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
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

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq)]
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

pub struct StateManager<S: EventSender> {
    event_sender: S,
    state: ThermostatState,
    config: Config,
    saved_target_temp: f32,
    last_idle_time: Instant,
}

impl<S: EventSender> StateManager<S> {
    pub fn new(config: &Config, state: ThermostatState, event_sender: S) -> Result<Self> {
        event_sender.send_event(
            Event::TimeoutReset(TimerId::Away, config.away_mode.timeout)
        )?;
        event_sender.send_event(
            Event::TimeoutReset(TimerId::Backlight, config.backlight.timeout)
        )?;

        Ok(Self {
            event_sender,
            state,
            config: config.clone(),
            saved_target_temp: 0.0,
            last_idle_time: Instant::now(),
        })
    }

    fn set_target_temp(&mut self, temp: f32) -> bool {
        if temp != self.state.target_temp {
            self.state.target_temp = temp;
            true
        } else {
            false
        }
    }

    fn set_current_temp(&mut self, temp: f32) -> bool {
        if temp != self.state.current_temp {
            self.state.current_temp = temp;
            true
        } else {
            false
        }
    }

    fn set_mode(&mut self, mode: HvacMode) -> bool {
        if mode != self.state.mode {
            self.state.mode = mode;
            true
        } else {
            false
        }
    }

    fn set_away(&mut self, is_away: bool) -> bool {
        if is_away != self.state.away {
            self.state.away = is_away;

            if self.state.away {
                self.saved_target_temp = self.state.target_temp;
                match self.state.mode {
                    HvacMode::Heat => {
                        self.state.target_temp = self.config.away_mode.temp_heat;
                    }
                    HvacMode::Cool => {
                        self.state.target_temp = self.config.away_mode.temp_cool;
                    }
                    _ => { }
                }
            } else {
                self.state.target_temp = self.saved_target_temp;
            }

            true
        } else {
            false
        }
    }

    fn apply_hvac_action(&mut self) -> bool {
        let old_action = self.state.action;

        if !self.state.backplate {
            self.state.action = HvacAction::Idle;
            return old_action != self.state.action;
        }

        let current_temp = self.state.current_temp;

        match self.state.mode {
            HvacMode::Heat => {
                let target_temp_hi = self.state.target_temp + self.config.temp_overrun;
                let target_temp_lo = self.state.target_temp - self.config.temp_deadband;

                if current_temp <= target_temp_lo {
                    self.state.action = HvacAction::Heating;
                } else if current_temp >= target_temp_hi {
                    self.state.action = HvacAction::Idle;
                }
            }
            HvacMode::Cool => {
                let target_temp_hi = self.state.target_temp + self.config.temp_deadband;
                let target_temp_lo = self.state.target_temp - self.config.temp_overrun;

                if current_temp >= target_temp_hi {
                    self.state.action = HvacAction::Cooling;
                } else if current_temp <= target_temp_lo {
                    self.state.action = HvacAction::Idle;
                }
            }
            HvacMode::Off => {
                self.state.action = HvacAction::Idle;
            }
            _ => { }
        };

        old_action != self.state.action
    }

    fn apply_lockout(&mut self) -> Result<()> {
        if self.state.action == HvacAction::Idle {
            // don't reset last idle time until min idle time elapsed
            // i.e. don't re-trigger lockout when it's already active
            if self.last_idle_time.elapsed() > self.config.min_off_time {
                self.last_idle_time = Instant::now();
            }

            self.state.lockout = None;
        } else {
            if self.last_idle_time.elapsed() < self.config.min_off_time {
                let lockout_time = self.config.min_off_time - self.last_idle_time.elapsed();
                self.state.lockout = Some(lockout_time);
                self.event_sender.send_event(
                    Event::StartTickTimer(TimerId::HvacLockout, lockout_time)
                )?;
            } else {
                self.state.lockout = None;
            }
        }

        Ok(())
    }
}

impl<S: EventSender> EventHandler for StateManager<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        let did_change = match event {
            Event::SetMode(mode) => {
                self.set_mode(*mode)
            }
            Event::SetTargetTemp(temp) => {
                self.set_target_temp(*temp)
            }
            Event::SetCurrentTemp(temp) => {
                self.set_current_temp(*temp)
            }
            Event::SetAway(false) | Event::ProximityNear | Event::ProximityFar | Event::Dial(_) => {
                self.event_sender.send_event(
                    Event::TimeoutReset(TimerId::Away, self.config.away_mode.timeout)
                )?;
                self.set_away(false)
            }
            Event::SetAway(true) | Event::TimeoutReached(TimerId::Away) => {
                self.set_away(true)
            }
            Event::TimeoutReached(TimerId::HvacLockout) => {
                self.state.lockout = None;
                true
            }
            Event::BackplateConnected => {
                self.state.backplate = true;
                true
            }
            Event::BackplateDisconnected => {
                self.state.backplate = false;
                true
            }
            _ => false
        };

        if did_change {
            if self.apply_hvac_action() {
                self.apply_lockout()?;
            }

            self.event_sender.send_event(Event::State(self.state.clone()))?;
        }

        if event.is_wakeup_event() {
            self.event_sender.send_event(
                Event::TimeoutReset(TimerId::Backlight, self.config.backlight.timeout)
            )?;
        }

        if event == &Event::GetState {
            self.event_sender.send_event(Event::State(self.state.clone()))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::Sender;

    use super::*;
    use crate::events::{DefaultEventSource, EventSource};

    fn state_manager(
        state: ThermostatState
    ) -> (DefaultEventSource, StateManager<Sender<Event>>)
    {
        let mut config = Config::default();
        config.temp_deadband = 0.4;
        config.temp_overrun = 0.2;

        let event_source = DefaultEventSource::new();
        let state_manager = StateManager::new(
            &config, state,
            event_source.event_sender()
        ).unwrap();

        (event_source, state_manager)
    }

    fn simulate<S>(
        mut state: StateManager<S>,
        steps: &[(f32, HvacAction)]
    ) -> Result<()>
        where S: EventSender
    {
        for (temp, action) in steps {
            state.handle_event(&Event::SetCurrentTemp(*temp))?;
            assert_eq!(
                state.state.action,
                *action,
                "temp {} expected {:?}, found {:?}",
                temp, *action, state.state.action
            );
        }

        Ok(())
    }

    #[test]
    fn temp_hysteresis_heat_on() -> Result<()> {
        let state = ThermostatState {
            mode: HvacMode::Heat,
            target_temp: 20.0,
            current_temp: 20.0,
            action: HvacAction::Idle,
            away: false,
            lockout: None,
            backplate: true,
        };

        let (_x, mgr) = state_manager(state);

        simulate(mgr, &[
            (20.0, HvacAction::Idle),
            (19.9, HvacAction::Idle),
            (19.8, HvacAction::Idle),
            (19.7, HvacAction::Idle),
            (19.6, HvacAction::Heating)
        ])
    }

    #[test]
    fn temp_hysteresis_heat_off() -> Result<()> {
        let state = ThermostatState {
            mode: HvacMode::Heat,
            target_temp: 20.0,
            current_temp: 20.0,
            action: HvacAction::Heating,
            away: false,
            lockout: None,
            backplate: true,
        };

        let (_x, mgr) = state_manager(state);

        simulate(mgr, &[
            (20.0, HvacAction::Heating),
            (20.1, HvacAction::Heating),
            (20.2, HvacAction::Idle)
        ])
    }

    #[test]
    fn temp_hysteresis_cool_on() -> Result<()> {
        let state = ThermostatState {
            mode: HvacMode::Cool,
            target_temp: 20.0,
            current_temp: 20.0,
            action: HvacAction::Idle,
            away: false,
            lockout: None,
            backplate: true,
        };

        let (_x, mgr) = state_manager(state);

        simulate(mgr, &[
            (20.0, HvacAction::Idle),
            (20.1, HvacAction::Idle),
            (20.2, HvacAction::Idle),
            (20.3, HvacAction::Idle),
            (20.4, HvacAction::Cooling)
        ])
    }

    #[test]
    fn temp_hysteresis_cool_off() -> Result<()> {
        let state = ThermostatState {
            mode: HvacMode::Cool,
            target_temp: 20.0,
            current_temp: 20.0,
            action: HvacAction::Cooling,
            away: false,
            lockout: None,
            backplate: true,
        };

        let (_x, mgr) = state_manager(state);

        simulate(mgr, &[
            (20.0, HvacAction::Cooling),
            (19.9, HvacAction::Cooling),
            (19.8, HvacAction::Idle)
        ])
    }

    #[test]
    fn min_off_time() -> Result<()> {
        let state = ThermostatState {
            mode: HvacMode::Cool,
            target_temp: 20.0,
            current_temp: 20.0,
            action: HvacAction::Idle,
            away: false,
            lockout: None,
            backplate: true,
        };

        let (_x, mut mgr) = state_manager(state);

        // idle -> cooling = lockout
        mgr.handle_event(&Event::SetCurrentTemp(21.0))?;
        assert!(mgr.state.action == HvacAction::Cooling);
        assert!(mgr.state.lockout.is_some());

        // lockout timer elapsed = no lockout
        mgr.handle_event(&Event::TimeoutReached(TimerId::HvacLockout))?;
        assert!(mgr.state.action == HvacAction::Cooling);
        assert!(mgr.state.lockout.is_none());

        // cooling -> idle = no lockout
        mgr.handle_event(&Event::SetCurrentTemp(19.0))?;
        assert!(mgr.state.action == HvacAction::Idle);
        assert!(mgr.state.lockout.is_none());

        // idle -> cooling = lockout
        mgr.handle_event(&Event::SetCurrentTemp(21.0))?;
        assert!(mgr.state.action == HvacAction::Cooling);
        assert!(mgr.state.lockout.is_some());

        // cooling -> idle = no lockout
        mgr.handle_event(&Event::SetCurrentTemp(19.0))?;
        assert!(mgr.state.action == HvacAction::Idle);
        assert!(mgr.state.lockout.is_none());

        // idle -> long delay -> cooling = no lockout
        mgr.last_idle_time = Instant::now() - Duration::from_mins(10);
        mgr.handle_event(&Event::SetCurrentTemp(21.0))?;
        assert!(mgr.state.action == HvacAction::Cooling);
        assert!(mgr.state.lockout.is_none());

        Ok(())
    }
}
