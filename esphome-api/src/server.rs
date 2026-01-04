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

use std::{io::BufReader, net::{TcpListener, TcpStream}};

use anyhow::Result;

use crate::{proto::*, proto_encrypted::EncryptedMessageStream, proto_plaintext::PlaintextMessageStream};

pub enum ResponseStatus {
    Continue,
    Disconnect
}

pub trait RequestHandler {
    fn handle_request<W: MessageWriter>(
        &self,
        message: &ProtoMessage,
        writer: &mut W
    ) -> Result<ResponseStatus>;
}

pub struct DefaultRequestHandler<D> {
    pub delegate: D,

    pub password: Option<String>
}

impl<D: RequestHandler> RequestHandler for DefaultRequestHandler<D> {
    fn handle_request<W: MessageWriter>(
        &self,
        message: &ProtoMessage,
        writer: &mut W
    ) -> Result<ResponseStatus> {
        match message {
            ProtoMessage::HelloRequest(_) => {
                writer.write(&HelloResponse {
                    // Mirrored API version from HA 2025.12.3
                    // This seems reasonable since that's what I'm developing against
                    api_version_major: 1,
                    api_version_minor: 13,
                    // I don't see server_info or name in HA dashboard anywhere
                    server_info: "My Server Info".to_string(),
                    name: "My Server Name".to_string()
                })?;
                Ok(ResponseStatus::Continue)
            }
            ProtoMessage::AuthenticationRequest(req) => {
                let invalid_password = if let Some(password) = &self.password {
                    *password != req.password
                } else {
                    false
                };

                writer.write(&AuthenticationResponse {
                    invalid_password: invalid_password
                })?;

                if invalid_password {
                    Ok(ResponseStatus::Disconnect)
                } else {
                    Ok(ResponseStatus::Continue)
                }
            }
            ProtoMessage::DisconnectRequest(_) => {
                writer.write(&DisconnectResponse { })?;
                Ok(ResponseStatus::Disconnect)
            }
            ProtoMessage::PingRequest(_) => {
                writer.write(&PingResponse { })?;
                Ok(ResponseStatus::Continue)
            }
            ProtoMessage::DeviceInfoRequest(_) => {
                writer.write(&DeviceInfoResponse {
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
                Ok(ResponseStatus::Continue)
            }
            message => self.delegate.handle_request(message, writer)
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
    let reader = BufReader::new(stream);

    let encrypted = true;
    let key = "jfD5V1SMKAPXNC8+d6BvE1EGBHJbyw2dSc0Q+ymNMhU=";
    let server_name = "My Device Name";
    let mac_addr = "00:00:00:00:00:01";

    if encrypted {
        let init = EncryptedMessageStream::init(reader, key, server_name, mac_addr)?;
        if let Some(stream) = init {
            message_loop(stream, handler)
        } else {
            // init() returns None when it needs to gracefully disconnect
            Ok(())
        }
    } else {
        let stream = PlaintextMessageStream::new(reader);
        message_loop(stream, handler)
    }
}

fn message_loop<S, H>(mut stream: S, handler: &H) -> Result<()>
    where S: MessageReader + MessageWriter, H: RequestHandler
{
    loop {
        let request = stream.read()?;
        println!("Request: {:?}", request);

        let status = handler.handle_request(&request, &mut stream)?;
        if matches!(status, ResponseStatus::Disconnect) {
            break;
        }
    }

    Ok(())
}
