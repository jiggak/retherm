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

use std::{io::BufReader, net::{TcpListener, TcpStream, ToSocketAddrs}, sync::{Arc, Mutex, mpsc::{Sender, channel}}, thread};

use anyhow::{Result, anyhow};
use base64::prelude::*;

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

pub trait MessageStreamProvider<S> {
    fn setup_stream(&self, stream: TcpStream) -> Result<S, ProtoError>;
}

pub struct PlaintextStreamProvider;

impl PlaintextStreamProvider {
    pub fn new() -> Self { Self }
}

impl MessageStreamProvider<PlaintextMessageStream> for PlaintextStreamProvider {
    fn setup_stream(&self, stream: TcpStream) -> Result<PlaintextMessageStream, ProtoError> {
        Ok(PlaintextMessageStream::new(BufReader::new(stream)))
    }
}

pub struct EncryptedStreamProvider {
    key: [u8; 32],
    node_name: String,
    mac_addr: String
}

impl EncryptedStreamProvider {
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

impl MessageStreamProvider<EncryptedMessageStream> for EncryptedStreamProvider {
    fn setup_stream(&self, stream: TcpStream) -> Result<EncryptedMessageStream, ProtoError> {
        let reader = BufReader::new(stream);
        let stream = EncryptedMessageStream::init(reader, &self.key, &self.node_name, &self.mac_addr)?;
        Ok(stream)
    }
}

pub trait ConnectionWatcher<S> {
    fn connected(&self, stream: &S) -> Result<()>;
    fn disconnect(&self) -> Result<()>;
}

#[derive(Clone)]
pub struct MessageSenderThread<M> {
    message_sender: Arc<Mutex<Option<Sender<M>>>>
}

impl<M: Message + MessageId + Send + 'static> MessageSenderThread<M> {
    pub fn new() -> Self {
        Self { message_sender: Arc::new(Mutex::new(None)) }
    }

    pub fn send_message(&self, message: M) -> Result<()> {
        let guard = self.message_sender.lock().unwrap();

        if let Some(sender) = guard.as_ref() {
            sender.send(message)?;
            Ok(())
        } else {
            Err(anyhow!("sender not initialized"))
        }
    }
}

impl<M: Message + MessageId + Send + 'static, S: MessageStream + Send + 'static> ConnectionWatcher<S> for MessageSenderThread<M> {
    fn connected(&self, stream: &S) -> Result<()> {
        let (tx, rx) = channel();

        *self.message_sender.lock().unwrap() = Some(tx);

        let mut stream = stream.clone();
        thread::spawn(move || {
            while let Ok(message) = rx.recv() {
                stream.write(&message).unwrap();
            }
        });

        Ok(())
    }

    fn disconnect(&self) -> Result<()> {
        *self.message_sender.lock().unwrap() = None;
        Ok(())
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
            ProtoMessage::AuthenticationRequest(_) => {
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
                writer.write(&DisconnectResponse::default())?;
                Ok(ResponseStatus::Disconnect)
            }
            ProtoMessage::PingRequest(_) => {
                writer.write(&PingResponse::default())?;
                Ok(ResponseStatus::Continue)
            }
            ProtoMessage::DeviceInfoRequest(_) => {
                // When I used values for response.project_*, HA would not show
                // any entities for the device
                let mut response = DeviceInfoResponse::default();

                response.name = self.node_name.clone();
                response.model = self.model.to_string();
                response.mac_address = self.mac_address.clone();
                response.manufacturer = self.manufacturer.clone();
                response.friendly_name = self.friendly_name.clone();
                // aioesphomeapi version for HA 2025.12.3 is 42.9.0
                // This shows as "Firmware" under device info in HA
                response.esphome_version = "42.9.0".to_string();

                writer.write(&response)?;
                Ok(ResponseStatus::Continue)
            }
            message => self.delegate.handle_request(message, writer)
        }
    }
}

pub fn start_server<S>(
    addr: impl ToSocketAddrs,
    stream_factory: &impl MessageStreamProvider<S>,
    watcher: &impl ConnectionWatcher<S>,
    handler: &impl RequestHandler
) -> Result<()>
    where S: MessageStream
{
    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming() {
        let stream = stream?;

        let message_stream = match stream_factory.setup_stream(stream) {
            // allow handshake disconnect to re-connect
            Err(ProtoError::HandshakeDisconnect) => continue,
            Err(error) => Err(error)?,
            Ok(stream) => stream
        };

        watcher.connected(&message_stream)?;

        message_loop(message_stream, handler)?;

        watcher.disconnect()?;
    }

    Ok(())
}

fn message_loop<S, H>(mut stream: S, handler: &H) -> Result<()>
    where S: MessageStream, H: RequestHandler
{
    loop {
        let request = stream.read()?;

        let status = handler.handle_request(&request, &mut stream)?;
        if matches!(status, ResponseStatus::Disconnect) {
            break;
        }
    }

    Ok(())
}
