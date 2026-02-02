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

use std::{sync::mpsc::{Receiver, Sender, channel}, thread, time::Duration};

use anyhow::Result;
use log::{debug, error};
use nest_backplate::{BackplateCmd, BackplateConnection, BackplateError, BackplateResponse, Wire};

use crate::{backplate::{HvacAction, HvacControl}, events::{Event, EventSender}};

pub struct DeviceBackplateThread {
    cmd_sender: Sender<BackplateCmd>
}

impl DeviceBackplateThread {
    const RECONNECT_TIMEOUT: Duration = Duration::from_secs(1);

    pub fn start<S>(dev_path: &'static str, event_sender: S) -> Result<Self>
        where S: EventSender + Send + 'static
    {
        let (cmd_sender, cmd_receiver) = channel();

        // Should I have spearate read/write threads?
        // With a single thread, I am relying on the backplate to send a message
        // before I can send one back. Maybe that's OK though, since the backplate
        // seems to constanty send messages.
        thread::spawn(move || {
            loop {
                match backplate_main_loop(dev_path, &event_sender, &cmd_receiver) {
                    Ok(()) => unreachable!("Backplate message loop should not return Ok"),
                    Err(error) => {
                        if let Some(error) = error.downcast_ref::<BackplateError>() {
                            if let BackplateError::IoError(error) = error {
                                error!(
                                    "Backplate thread IoError `{}`, reconnect in {:?}",
                                    error, Self::RECONNECT_TIMEOUT
                                );
                                thread::sleep(Self::RECONNECT_TIMEOUT);
                                continue;
                            }
                        }

                        Err::<(), anyhow::Error>(error)
                    }
                }.expect("Backplate thread error");
            }
        });

        Ok(Self {
            cmd_sender
        })
    }
}

fn backplate_main_loop<S: EventSender>(
    dev_path: &str,
    event_sender: &S,
    cmd_receiver: &Receiver<BackplateCmd>
) -> Result<()> {
    let mut backplate = BackplateConnection::open(dev_path)?;

    // This triggers a constant stream of messages
    backplate.send_command(BackplateCmd::StatusRequest)?;

    loop {
        match backplate.read_message()? {
            BackplateResponse::Climate(c) => {
                event_sender.send_event(Event::SetCurrentTemp(c.temperature))?;
            }
            BackplateResponse::WireSwitched(_wire, _state) => {
                // FIXME I sort of "set it and forget it" with the hvac
                // action. Seems like a good idea to do something with this
                // message to confirm the state changed somehow.
                // println!("Wire:{:?} state:{}", wire, state);
            }
            msg => {
                debug!("{:?}", msg);
            }
        }

        if let Ok(cmd) = cmd_receiver.try_recv() {
            backplate.send_command(cmd)?;
        }
    }
}

impl HvacControl for DeviceBackplateThread {
    fn switch_hvac(&self, action: &HvacAction) -> Result<()> {
        let cmds = match action {
            HvacAction::Heating => {
                [
                    BackplateCmd::SwitchWire(Wire::W1, true),
                    BackplateCmd::SwitchWire(Wire::Y1, false)
                ]
            }
            HvacAction::Cooling => {
                [
                    BackplateCmd::SwitchWire(Wire::W1, false),
                    BackplateCmd::SwitchWire(Wire::Y1, true)
                ]
            }
            HvacAction::Idle => {
                [
                    BackplateCmd::SwitchWire(Wire::W1, false),
                    BackplateCmd::SwitchWire(Wire::Y1, false)
                ]
            }
        };

        for cmd in cmds {
            self.cmd_sender.send(cmd)?;
        }

        Ok(())
    }
}
