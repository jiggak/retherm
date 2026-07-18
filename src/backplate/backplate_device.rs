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

use std::{
    sync::{Arc, Mutex, mpsc::{Receiver, Sender, channel}},
    thread,
    time::{Duration, Instant}
};

use anyhow::Result;
use log::{debug, error, info, warn};
use nest_backplate::{BackplateCmd, BackplateConnection, BackplateResponse, Wire};

use crate::{
    config::{BackplateConfig, Config, WireConfig, WireId},
    events::{Event, EventSender},
    state::HvacAction
};
use super::{BackplateDevice};

pub struct DeviceBackplateThread {
    cmd_sender: Sender<BackplateCmd>,
    wire_state: Arc<Mutex<SwitchState>>,
}

impl DeviceBackplateThread {
    const RECONNECT_TIMEOUT: Duration = Duration::from_secs(1);
    const KEEPALIVE_PERIOD: Duration = Duration::from_mins(15);

    pub fn start<S>(config: BackplateConfig, event_sender: S) -> Result<Self>
        where S: EventSender + Send + 'static
    {
        let (cmd_sender, cmd_receiver) = channel();
        let serial_port = config.serial_port.clone();
        let near_pir_threshold = config.near_pir_threshold;

        let wire_state = match config.wiring {
            WireConfig::HeatAndCool { heat_wire, cool_wire, fan_wire } => {
                SwitchState::new(heat_wire.into(), cool_wire.into(), fan_wire.into())
            }
        };
        let wire_state = Arc::new(Mutex::new(wire_state));
        let wire_state_clone = wire_state.clone();

        // Should I have spearate read/write threads?
        // With a single thread, I am relying on the backplate to send a message
        // before I can send one back. Maybe that's OK though, since the backplate
        // seems to constanty send messages.
        thread::spawn(move || {
            loop {
                // drain cmd_receiver incase cmds sent while disconnected
                while let Ok(_) = cmd_receiver.try_recv() { }

                // reset back to "Idle" since that's the state on backplate connect
                wire_state.lock().unwrap().clear();

                let result = backplate_main_loop(
                    &serial_port,
                    near_pir_threshold,
                    Self::KEEPALIVE_PERIOD,
                    &event_sender,
                    &cmd_receiver,
                    &wire_state
                );

                match result {
                    Ok(_) => unreachable!("Backplate message loop should not return Ok"),
                    Err(error) => {
                        event_sender.send_event(Event::BackplateDisconnected).unwrap();

                        error!(
                            "Backplate thread error `{}`, reconnect in {:?}",
                            error, Self::RECONNECT_TIMEOUT
                        );

                        thread::sleep(Self::RECONNECT_TIMEOUT);
                    }
                }
            }
        });

        Ok(Self {
            cmd_sender,
            wire_state: wire_state_clone,
        })
    }
}

fn backplate_main_loop<S: EventSender>(
    dev_path: &str,
    near_pir_threshold: u16,
    keepalive_period: Duration,
    event_sender: &S,
    cmd_receiver: &Receiver<BackplateCmd>,
    wire_state: &Arc<Mutex<SwitchState>>
) -> Result<()> {
    let mut backplate = BackplateConnection::open(dev_path)?;

    event_sender.send_event(Event::BackplateConnected)?;

    // Log backplate version details
    backplate.send_command(BackplateCmd::GetTfeBuildInfo)?;

    // This triggers a constant stream of messages
    backplate.send_command(BackplateCmd::StatusRequest)?;
    let mut last_status_request = Instant::now();

    loop {
        match backplate.read_message()? {
            BackplateResponse::Climate(c) => {
                event_sender.send_event(Event::SetCurrentTemp(c.temperature))?;
            }
            BackplateResponse::NearPir(val) => {
                if val > near_pir_threshold {
                    event_sender.send_event(Event::ProximityNear)?;
                }
            }
            BackplateResponse::Pir { val1, val2 } => {
                if val1 + val2 > 0 {
                    event_sender.send_event(Event::ProximityFar)?;
                }
            }
            BackplateResponse::WireSwitched(wire, state) => {
                info!("WireSwitched {wire:?}: {state}");
                wire_state.lock().unwrap().set_wire_state(wire, state);
            }
            BackplateResponse::TfeBuildInfo(s) => {
                info!("{}", s);
            }
            // BackplateResponse::AmbientLightSensor(_) => { }
            // BackplateResponse::Raw(Message { command_id: 19, .. }) => { }
            x if x.is_break() => {
                warn!("Break received, resetting");
                backplate.reset_ack()?;

                // Resume message stream
                backplate.send_command(BackplateCmd::StatusRequest)?;

                // Restore wire state switches
                for cmd in wire_state.lock().unwrap().commands() {
                    backplate.send_command(cmd)?;
                }
            }
            msg => {
                debug!("{:?}", msg);
            }
        }

        if let Ok(cmd) = cmd_receiver.try_recv() {
            backplate.send_command(cmd)?;
        }

        // Nest will reboot itself 30min after starting backplate comms.
        // I don't know specifically what mechanism causes this, but
        // sending periodic StatusRequest message prevents reboot.
        if Instant::now() - last_status_request > keepalive_period {
            info!("Sending StatusRequest for keepalive");
            backplate.send_command(BackplateCmd::StatusRequest)?;
            last_status_request = Instant::now();
        }
    }
}

impl BackplateDevice for DeviceBackplateThread {
    fn new<S>(config: &Config, event_sender: S) -> Result<Self>
        where S: EventSender + Send + 'static, Self: Sized
    {
        DeviceBackplateThread::start(
            config.backplate.clone(),
            event_sender
        )
    }

    fn switch_hvac(&self, action: &HvacAction) -> Result<()> {
        let state = self.wire_state.lock().unwrap();

        if !state.is_active(action) {
            for cmd in state.switch_commands(action) {
                self.cmd_sender.send(cmd)?;
            }
        }

        Ok(())
    }
}

impl From<WireId> for Wire {
    fn from(value: WireId) -> Self {
        match value {
            WireId::W1 => Self::W1,
            WireId::Y1 => Self::Y1,
            WireId::G => Self::G,
            WireId::OB => Self::OB,
            WireId::W2 => Self::W2,
            WireId::Y2 => Self::Y2,
            WireId::Star => Self::Star
        }
    }
}

struct SwitchState {
    heat_wire: (Wire, bool),
    cool_wire: (Wire, bool),
    fan_wire: (Wire, bool),
}

impl SwitchState {
    fn new(heat_wire: Wire, cool_wire: Wire, fan_wire: Wire) -> Self {
        Self {
            heat_wire: (heat_wire, false),
            cool_wire: (cool_wire, false),
            fan_wire: (fan_wire, false),
        }
    }

    fn commands(&self) -> [BackplateCmd; 3] {
        [
            BackplateCmd::SwitchWire(self.heat_wire.0, self.heat_wire.1),
            BackplateCmd::SwitchWire(self.cool_wire.0, self.cool_wire.1),
            BackplateCmd::SwitchWire(self.fan_wire.0, self.fan_wire.1),
        ]
    }

    fn switch_commands(&self, action: &HvacAction) -> [BackplateCmd; 3] {
        match action {
            HvacAction::Heating => {
                [
                    BackplateCmd::SwitchWire(self.heat_wire.0, true),
                    BackplateCmd::SwitchWire(self.cool_wire.0, false),
                    BackplateCmd::SwitchWire(self.fan_wire.0, false),
                ]
            }
            HvacAction::Cooling => {
                [
                    BackplateCmd::SwitchWire(self.heat_wire.0, false),
                    BackplateCmd::SwitchWire(self.cool_wire.0, true),
                    BackplateCmd::SwitchWire(self.fan_wire.0, false),
                ]
            }
            HvacAction::Fan => {
                [
                    BackplateCmd::SwitchWire(self.heat_wire.0, false),
                    BackplateCmd::SwitchWire(self.cool_wire.0, false),
                    BackplateCmd::SwitchWire(self.fan_wire.0, true),
                ]
            }
            HvacAction::Idle => {
                [
                    BackplateCmd::SwitchWire(self.heat_wire.0, false),
                    BackplateCmd::SwitchWire(self.cool_wire.0, false),
                    BackplateCmd::SwitchWire(self.fan_wire.0, false),
                ]
            }
        }
    }

    fn is_active(&self, action: &HvacAction) -> bool {
        match action {
            HvacAction::Heating => self.heat_wire.1,
            HvacAction::Cooling => self.cool_wire.1,
            HvacAction::Fan => self.fan_wire.1,
            HvacAction::Idle => {
                !self.cool_wire.1 && !self.heat_wire.1 && !self.fan_wire.1
            }
        }
    }

    fn set_wire_state(&mut self, wire: Wire, val: bool) {
        if wire == self.cool_wire.0 {
            self.cool_wire.1 = val;
        } else if wire == self.heat_wire.0 {
            self.heat_wire.1 = val;
        } else if wire == self.fan_wire.0 {
            self.fan_wire.1 = val;
        } else {
            panic!("Unexpected wire {:?}", wire);
        }
    }

    fn clear(&mut self) {
        self.heat_wire.1 = false;
        self.cool_wire.1 = false;
        self.fan_wire.1 = false;
    }
}
