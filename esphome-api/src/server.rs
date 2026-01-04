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

use anyhow::{Result, anyhow};
use base64::prelude::*;

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

        handle_connection_encrypted(&handler, stream)?;
    }

    Ok(())
}

fn handle_connection_encrypted<H: RequestHandler>(handler: &H, stream: TcpStream) -> Result<()> {
    let noise_psk = BASE64_STANDARD.decode("jfD5V1SMKAPXNC8+d6BvE1EGBHJbyw2dSc0Q+ymNMhU=")?;
    let noise_psk: [u8; 32] = noise_psk.try_into().unwrap();

    let mut noise = snow::Builder::new("Noise_NNpsk0_25519_ChaChaPoly_SHA256".parse()?)
        // do I need prologue?
        .prologue(b"NoiseAPIInit\0\0")?
        .psk(0, &noise_psk)?
        .build_responder()?;

    let mut reader = BufReader::new(stream);

    // https://developers.esphome.io/architecture/api/protocol_details/

    let frame1 = match read_encrypted_frame(&mut reader) {
        Err(ProtoError::InvalidIndicator(1, 0)) => {
            write_handshake_reject(&mut reader.get_ref(), "Bad indicator byte")?;
            println!("Sent invalid frame to client... disconnect");
            return Ok(());
        }
        r => r
    }?;

    // First frame is NOISE_HELLO; zero length
    println!("Frame1 {:02x?}", &frame1[..]);
    if frame1.len() > 0 {
        return Err(anyhow!("I expected first frame after connect to be zero length"));
    }

    write_hello_frame(&mut reader.get_ref(), "test_device", "00:00:00:00:00:01")?;

    let frame2 = read_encrypted_frame(&mut reader)?;

    // TODO is static buffer necessary?
    let mut buffer = vec![0u8; 512];
    // let mut buffer = BytesMut::new();
    match noise.read_message(&frame2[1..], &mut buffer) {
        Err(snow::Error::Decrypt) => {
            write_handshake_reject(&mut reader.get_ref(), "Handshake MAC failure")?;
            println!("Sent handshake failed to client... disconnect");
            return Ok(());
        }
        r => r
    }?;

    // let mut buffer = BytesMut::new();
    let len = noise.write_message(&[], &mut buffer)?;
    println!("Noise write {}", len);

    let mut payload = vec![0x00];
    payload.extend_from_slice(&buffer[..len]);

    write_encrypted_frame(&mut reader.get_ref(), &payload)?;
    println!("Sent handshake success");

    let mut transport = noise.into_transport_mode()?;
    loop {
        let request = read_message_encrypted(&mut reader, &mut transport)?;
        println!("Request: {:?}", request);

        let mut stream = reader.get_ref();

        match request {
            ProtoMessage::HelloRequest(_) => {
                write_encrypted_message(&mut stream, &mut transport, &HelloResponse {
                    // Mirrored API version from HA 2025.12.3
                    // This seems reasonable since that's what I'm developing against
                    api_version_major: 1,
                    api_version_minor: 13,
                    // I don't see server_info or name in HA dashboard anywhere
                    server_info: "My Server Info".to_string(),
                    name: "My Server Name".to_string()
                })?
            }
            _ => { }
        }
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
