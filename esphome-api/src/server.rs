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

use std::{io::BufReader, net::{TcpListener, TcpStream, ToSocketAddrs}};

use base64::prelude::*;
use anyhow::{Result, anyhow};

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

pub trait MessageStreamFactory<S> {
    fn new_stream(&self, stream: TcpStream) -> Result<S>;
}

pub struct PlaintextMessageStreamFactory;

impl PlaintextMessageStreamFactory {
    pub fn new() -> Self { Self }
}

impl MessageStreamFactory<PlaintextMessageStream> for PlaintextMessageStreamFactory {
    fn new_stream(&self, stream: TcpStream) -> Result<PlaintextMessageStream> {
        Ok(PlaintextMessageStream::new(BufReader::new(stream)))
    }
}

pub struct EncryptedMessageStreamFactory {
    key: [u8; 32],
    node_name: String,
    mac_addr: String
}

impl EncryptedMessageStreamFactory {
    pub fn new(key: &str, node_name: &str, mac_addr: &str) -> Result<Self> {
        let key_bytes = BASE64_STANDARD.decode(key)?;
        let key: [u8; 32] = key_bytes.try_into()
            .map_err(|_| anyhow!("Key must be 32 bytes"))?;

        Ok(Self {
            key,
            node_name: node_name.to_string(),
            mac_addr: mac_addr.to_string()
        })
    }
}

impl MessageStreamFactory<EncryptedMessageStream> for EncryptedMessageStreamFactory {
    fn new_stream(&self, stream: TcpStream) -> Result<EncryptedMessageStream> {
        let reader = BufReader::new(stream);
        let stream = EncryptedMessageStream::init(reader, &self.key, &self.node_name, &self.mac_addr)?;
        Ok(stream)
    }
}

pub struct DefaultHandler<D> {
    pub delegate: D,

    pub server_info: String,
    pub node_name: String,
    pub friendly_name: String,
    pub manufacturer: String,
    pub model: String,
    pub mac_address: String
}

impl<D: RequestHandler> RequestHandler for DefaultHandler<D> {
    fn handle_request<W: MessageWriter>(
        &self,
        message: &ProtoMessage,
        writer: &mut W
    ) -> Result<ResponseStatus> {
        match message {
            ProtoMessage::HelloRequest(_) => {
                writer.write(&HelloResponse {
                    // HA 2025.12.3 is what I'm using for development
                    // It reports 1.13, so it probably makes sense to mirror it?
                    // aioesphomeapi/connection.py confirms this version too
                    api_version_major: 1,
                    api_version_minor: 13,
                    // I don't see server_info or name in HA dashboard anywhere
                    server_info: self.server_info.to_string(),
                    name: self.node_name.clone(),
                })?;
                Ok(ResponseStatus::Continue)
            }
            ProtoMessage::AuthenticationRequest(req) => {
                // As of HA 2026.1.0 password auth is removed
                // Apparently, these messages will no longer be used

                writer.write(&AuthenticationResponse {
                    invalid_password: false
                })?;

                if false { // Disconnect when password invalid
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
                    uses_password: false,
                    name: self.node_name.clone(),
                    mac_address: self.mac_address.clone(),
                    // aioesphomeapi version for HA 2025.12.3 is 42.9.0
                    // This shows as "Firmware" under device info in HA
                    esphome_version: "42.9.0".to_string(),
                    compilation_time: "".to_string(),
                    model: self.model.to_string(),
                    has_deep_sleep: false,
                    // When I used values for project_*, HA would not show
                    // any entities for the device
                    project_name: "".to_string(),
                    project_version: "".to_string(),
                    webserver_port: 0,
                    #[allow(deprecated)] legacy_bluetooth_proxy_version: 0,
                    bluetooth_proxy_feature_flags: 0,
                    manufacturer: self.manufacturer.clone(),
                    friendly_name: self.friendly_name.clone(),
                    #[allow(deprecated)] legacy_voice_assistant_version: 0,
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

pub fn start_server<A, F, S, H>(addr: A, stream_factory: &F, handler: &H) -> Result<()>
    where A: ToSocketAddrs, H: RequestHandler, S: MessageStream, F: MessageStreamFactory<S>
{
    let listener = TcpListener::bind(addr)?;

    println!("Listen for incoming");
    for stream in listener.incoming() {
        let stream = stream?;

        println!("Connection established");

        match stream_factory.new_stream(stream) {
            Ok(stream) => message_loop(stream, handler)?,
            Err(error) => {
                match error.downcast_ref::<ProtoError>() {
                    // allow handshake disconnect to re-connect
                    Some(ProtoError::HandshakeDisconnect) => continue,
                    _ => return Err(error)
                }
            }
        }
    }

    Ok(())
}

fn message_loop<S, H>(mut stream: S, handler: &H) -> Result<()>
    where S: MessageStream, H: RequestHandler
{
    loop {
        let request = stream.read()?;
        println!("Request {:?}", request);

        let status = handler.handle_request(&request, &mut stream)?;
        if matches!(status, ResponseStatus::Disconnect) {
            break;
        }
    }

    Ok(())
}
