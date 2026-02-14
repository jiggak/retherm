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

use crate::{events::{Event, EventHandler, EventSender}, state::HvacAction};

#[cfg(feature = "device")]
mod backplate_device;

#[cfg(feature = "device")]
use backplate_device::DeviceBackplateThread as BackplateImpl;

#[cfg(feature = "simulate")]
mod backplate_simulated;

#[cfg(feature = "simulate")]
use backplate_simulated::SimulatedBackplate as BackplateImpl;

trait BackplateDevice {
    fn new<S>(event_sender: S) -> Result<Self>
        where S: EventSender + Send + 'static, Self: Sized;

    fn switch_hvac(&self, action: &HvacAction) -> Result<()>;
}

pub struct Backplate<D> {
    device: D
}

impl Backplate<BackplateImpl> {
    pub fn new<S>(event_sender: S) -> Result<Self>
        where S: EventSender + Send + 'static
    {
        let device = BackplateImpl::new(event_sender)?;
        Ok(Self { device })
    }
}

impl<D: BackplateDevice> EventHandler for Backplate<D> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if let Event::State(state) = event {
            // TODO do I need some sort of "cooldown" phase if circuit was
            // goes from on to off then on again too quickly?
            self.device.switch_hvac(&state.action)?;
        }

        Ok(())
    }
}
