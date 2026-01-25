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

use std::{sync::mpsc::{Sender, channel}, thread::{self, JoinHandle}};

use anyhow::Result;
use nest_backplate::{BackplateCmd, BackplateConnection, BackplateResponse, Wire};

use crate::{backplate::{HvacAction, HvacControl}, events::{Event, EventSender}};

pub struct DeviceBackplateThread {
    handle: JoinHandle<Result<()>>,
    cmd_sender: Sender<ThreadCmd>
}

impl DeviceBackplateThread {
    pub fn start<S>(dev_path: &str, event_sender: S) -> Result<Self>
        where S: EventSender + Send + 'static
    {
        let (cmd_sender, cmd_receiver) = channel();

        let mut backplate = BackplateConnection::open(dev_path)?;

        // This triggers a constant stream of messages
        backplate.send_command(BackplateCmd::StatusRequest)?;

        // Should I have spearate read/write threads?
        // With a single thread, I am relying on the backplate to send a message
        // before I can send one back. Maybe that's OK though, since the backplate
        // seems to constanty send message wh
        let handle = thread::spawn(move || {
            loop {
                match backplate.read_message()? {
                    BackplateResponse::Climate { temperature, .. } => {
                        event_sender.send_event(Event::SetCurrentTemp(temperature))?;
                    }
                    _ => { }
                }

                if let Ok(cmd) = cmd_receiver.try_recv() {
                    match cmd {
                        ThreadCmd::Stop => break,
                        ThreadCmd::Backplate(cmd) => {
                            backplate.send_command(cmd)?;
                        }
                    }
                }
            }

            Ok(())
        });

        Ok(Self {
            handle, cmd_sender
        })
    }

    pub fn stop(self) -> Result<()> {
        self.cmd_sender.send(ThreadCmd::Stop)?;
        self.handle.join().unwrap()
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
            self.cmd_sender.send(ThreadCmd::Backplate(cmd))?;
        }

        Ok(())
    }
}

pub enum ThreadCmd {
    Backplate(BackplateCmd),
    Stop
}
