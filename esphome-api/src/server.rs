/*
 * Nest UI - Home Assistant native thermostat interface
 * Copyright (C) 2025 Josh Kropf <josh@slashdev.ca>
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

use std::{io::{BufReader, Write}, net::{TcpListener, TcpStream}};

use anyhow::Result;

use crate::proto::*;

pub enum ConnectStatus {
    Continue,
    Disconnect
}

pub trait RequestHandler {
    fn handle_request<S: Write>(&self, message: &ProtoMessage, stream: &mut S) -> Result<ConnectStatus>;
}

pub struct DefaultRequestHandler<D> {
    pub delegate: D,

    pub password: Option<String>
}

impl<D: RequestHandler> RequestHandler for DefaultRequestHandler<D> {
    fn handle_request<S: Write>(&self, message: &ProtoMessage, stream: &mut S) -> Result<ConnectStatus> {
        match message {
            ProtoMessage::HelloRequest(_) => {
                write_message(stream, &HelloResponse {
                    // Mirrored API version from HA 2025.12.3
                    // This seems reasonable since that's what I'm developing against
                    api_version_major: 1,
                    api_version_minor: 13,
                    // I don't see server_info or name in HA dashboard anywhere
                    server_info: "My Server Info".to_string(),
                    name: "My Server Name".to_string()
                })?;
                Ok(ConnectStatus::Continue)
            }
            ProtoMessage::AuthenticationRequest(req) => {
                let invalid_password = if let Some(password) = &self.password {
                    *password != req.password
                } else {
                    false
                };

                write_message(stream, &AuthenticationResponse {
                    invalid_password: invalid_password
                })?;

                if invalid_password {
                    Ok(ConnectStatus::Disconnect)
                } else {
                    Ok(ConnectStatus::Continue)
                }
            }
            ProtoMessage::DisconnectRequest(_) => {
                write_message(stream, &DisconnectResponse { })?;
                Ok(ConnectStatus::Disconnect)
            }
            ProtoMessage::PingRequest(_) => {
                write_message(stream, &PingResponse { })?;
                Ok(ConnectStatus::Continue)
            }
            ProtoMessage::DeviceInfoRequest(_) => {
                write_message(stream, &DeviceInfoResponse {
                    uses_password: self.password.is_some(),
                    name: "My Device Name".to_string(),
                    mac_address: "00:00:00:00:00:01".to_string(),
                    esphome_version: "2025.12.2".to_string(),
                    compilation_time: "".to_string(),
                    model: "My Device Model".to_string(),
                    has_deep_sleep: false,
                    // When I used values for project_*, HA would not show
                    // any entities for the device
                    project_name: "".to_string(),
                    project_version: "".to_string(),
                    webserver_port: 0,
                    legacy_bluetooth_proxy_version: 0,
                    bluetooth_proxy_feature_flags: 0,
                    manufacturer: "Josh".to_string(),
                    friendly_name: "My Device Friendly Name".to_string(),
                    legacy_voice_assistant_version: 0,
                    voice_assistant_feature_flags: 0,
                    suggested_area: "".to_string(),
                    bluetooth_mac_address: "".to_string(),
                    api_encryption_supported: false,
                    devices: vec![],
                    areas: vec![],
                    area: None,
                    zwave_proxy_feature_flags: 0,
                    zwave_home_id: 0
                })?;
                Ok(ConnectStatus::Continue)
            }
            _ => self.delegate.handle_request(message, stream)
        }
    }
}

pub fn start_server<H: RequestHandler>(handler: H) -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:6053")?;

    println!("Listen for incoming");
    for stream in listener.incoming() {
        println!("Connection established");
        let stream = stream?;

        handle_connection(&handler, stream)?;
    }

    Ok(())
}

fn handle_connection<H: RequestHandler>(handler: &H, stream: TcpStream) -> Result<()> {
    let mut reader = BufReader::new(stream);

    loop {
        // the robot suggested `AsyncReadExt::read_buf` from tokio to read straight
        // into the `BytesMut` instance. That looks way cleaner... might be time to
        // stop resisting tokio.
        // e.g. stream.read_buf(&mut message_buffer)?;

        let request = read_message(&mut reader)?;
        println!("Request: {:?}", request);

        let mut stream = reader.get_ref();

        let status = handler.handle_request(&request, &mut stream)?;
        if matches!(status, ConnectStatus::Disconnect) {
            break;
        }
    }

    Ok(())
}
