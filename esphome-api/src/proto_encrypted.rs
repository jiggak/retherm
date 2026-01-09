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

use std::{io::{BufRead, BufReader, Write}, net::TcpStream};

use anyhow::Result;
use prost::{Message, bytes::{Buf, BufMut, Bytes, BytesMut}};
use snow::TransportState;

use crate::proto::{MessageId, MessageReader, MessageStream, MessageWriter, ProtoError, ProtoMessage};

pub struct EncryptedMessageStream {
    reader: BufReader<TcpStream>,
    codec: TransportState
}

// References for the encrypted connection setup:
// https://developers.esphome.io/architecture/api/protocol_details/
// https://ubihome.github.io/esphome-native-api/native_api/encryption/

impl EncryptedMessageStream {
    pub fn init(
        mut reader: BufReader<TcpStream>,
        key: &[u8; 32],
        node_name: &str,
        mac_addr: &str
    ) -> Result<Self> {
        let mut noise = snow::Builder::new("Noise_NNpsk0_25519_ChaChaPoly_SHA256".parse()?)
            // do I need prologue?
            .prologue(b"NoiseAPIInit\0\0")?
            .psk(0, &key)?
            .build_responder()?;

        let frame1 = match read_encrypted_frame(&mut reader) {
            Err(ProtoError::InvalidIndicator(1, 0)) => {
                write_handshake_reject(&mut reader.get_ref(), "Bad indicator byte")?;
                Err(ProtoError::HandshakeDisconnect)
            }
            r => r
        }?;

        // First frame is NOISE_HELLO; zero length
        if frame1.len() > 0 {
            return Err(ProtoError::ExpectedNoiseHello.into());
        }

        write_hello_frame(&mut reader.get_ref(), node_name, mac_addr)?;

        let frame2 = read_encrypted_frame(&mut reader)?;

        // TODO is static buffer necessary?
        let mut buffer = vec![0u8; 512];
        // let mut buffer = BytesMut::new();
        match noise.read_message(&frame2[1..], &mut buffer) {
            Err(snow::Error::Decrypt) => {
                write_handshake_reject(&mut reader.get_ref(), "Handshake MAC failure")?;
                Err(ProtoError::HandshakeDisconnect)
            }
            r => r.map_err(|e| ProtoError::from(e))
        }?;

        // let mut buffer = BytesMut::new();
        let len = noise.write_message(&[], &mut buffer)?;

        let mut payload = vec![0x00];
        payload.extend_from_slice(&buffer[..len]);

        write_encrypted_frame(&mut reader.get_ref(), &payload)?;

        let codec = noise.into_transport_mode()?;

        Ok(Self { reader, codec })
    }
}

impl MessageStream for EncryptedMessageStream { }

impl MessageReader for EncryptedMessageStream {
    fn read(&mut self) -> Result<ProtoMessage, ProtoError> {
        let frame = read_encrypted_frame(&mut self.reader)?;

        let mut buffer = vec![0u8; 512];
        let len = self.codec.read_message(&frame, &mut buffer)?;

        let mut buffer = Bytes::copy_from_slice(&buffer[..len]);

        let message_type = buffer.get_u16() as u64;
        let _message_size = buffer.get_u16();

        Ok(ProtoMessage::decode(message_type, &mut buffer)?)
    }
}

impl MessageWriter for EncryptedMessageStream {
    fn write<M>(&mut self, message: &M) -> Result<()>
        where M: Message + MessageId
    {
        let mut message_buffer = BytesMut::with_capacity(512);
        encode_message(message, &mut message_buffer)?;

        let buf = message_buffer.freeze();

        let mut buffer = vec![0u8; 512];
        let len = self.codec.write_message(&buf, &mut buffer)?;

        write_encrypted_frame(&mut self.reader.get_ref(), &buffer[..len])?;

        Ok(())
    }
}

fn encode_message<M, B>(message: &M, buffer: &mut B) -> Result<()>
    where M: Message + MessageId, B: BufMut
{
    let message_id = M::ID as u16;
    let message_len = message.encoded_len() as u16;

    buffer.put_u16(message_id);
    buffer.put_u16(message_len);
    message.encode(buffer)?;

    Ok(())
}

fn read_encrypted_frame<R: BufRead>(stream: &mut R) -> Result<Bytes, ProtoError> {
    let buf = stream.fill_buf()?;
    if buf.len() == 0 {
        return Err(ProtoError::ReadZero);
    }

    let mut buffer = Bytes::copy_from_slice(buf);

    let byte_zero = buffer.get_u8();
    if byte_zero != 1 {
        return Err(ProtoError::InvalidIndicator(1, byte_zero));
    }

    let message_size = buffer.get_u16() as usize;
    stream.consume(3);

    let buf = stream.fill_buf()?;
    if buf.len() < message_size {
        return Err(ProtoError::BufferUnderrun(buf.len(), message_size));
    }

    let buffer = Bytes::copy_from_slice(&buf[..message_size]);
    stream.consume(message_size);

    Ok(buffer)
}

fn write_encrypted_frame<S: Write>(stream: &mut S, payload: &[u8]) -> Result<()> {
    let mut buffer = BytesMut::new();

    buffer.put_u8(1);
    buffer.put_u16(payload.len() as u16);
    buffer.put(payload);

    let buf = buffer.freeze();
    stream.write_all(&buf)?;

    Ok(())
}

fn write_handshake_reject<S: Write>(stream: &mut S, reason: &str) -> Result<()> {
    let mut payload = vec![0x01];
    payload.extend_from_slice(reason.as_bytes());

    write_encrypted_frame(stream, payload.as_slice())
}

fn write_hello_frame<S: Write>(stream: &mut S,  node_name: &str, mac_addr: &str) -> Result<()> {
    let mut payload = vec![0x01];
    payload.extend_from_slice(node_name.as_bytes());
    payload.push(0);
    payload.extend_from_slice(mac_addr.as_bytes());
    payload.push(0);

    write_encrypted_frame(stream, payload.as_slice())
}
