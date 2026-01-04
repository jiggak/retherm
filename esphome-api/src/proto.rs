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

include!(concat!(env!("OUT_DIR"), "/esphome_proto.rs"));
include!(concat!(env!("OUT_DIR"), "/message_ids.rs"));
include!(concat!(env!("OUT_DIR"), "/proto_message.rs"));

use std::io::{BufRead, Write};

use anyhow::{Result, anyhow};
use prost::{Message, bytes::{Buf, BufMut, Bytes, BytesMut}, encoding::{decode_varint, encode_varint}};
use snow::TransportState;

#[derive(Debug)]
struct Frame {
    message_size: u64,
    type_id: u64
}

impl Frame {
    pub fn decode<B: Buf>(buffer: &mut B) -> Result<Self> {
        let byte_zero = buffer.get_u8();
        if byte_zero != 0 {
            return Err(anyhow!("Expected first byte of frame to be zero, found {}", byte_zero));
        }

        let message_size = decode_varint(buffer)?;
        let type_id = decode_varint(buffer)?;

        Ok(Self {
            message_size, type_id
        })
    }
}

pub trait MessageId {
    const ID: u64;
}

fn encode_message<M, B>(message: &M, buffer: &mut B) -> Result<()>
    where M: Message + MessageId, B: BufMut
{
    let message_id = M::ID;
    let message_len = message.encoded_len();

    buffer.put_u8(0u8);
    encode_varint(message_len as u64, buffer);
    encode_varint(message_id, buffer);
    message.encode(buffer)?;

    Ok(())
}

fn encode_encrypted_message<M, B>(message: &M, buffer: &mut B) -> Result<()>
    where M: Message + MessageId, B: BufMut
{
    let message_id = M::ID as u16;
    let message_len = message.encoded_len() as u16;

    buffer.put_u16(message_id);
    buffer.put_u16(message_len);
    message.encode(buffer)?;

    Ok(())
}

pub fn read_message<R: BufRead>(stream: &mut R) -> Result<ProtoMessage> {
    let buf = stream.fill_buf()?;
    if buf.len() == 0 {
        return Err(anyhow!("Read zero bytes expecting frame"));
    }

    let mut buffer = Bytes::copy_from_slice(buf);
    println!("Frame buffer {} - {:02x?}", buf.len(), buf);

    let frame = Frame::decode(&mut buffer)?;
    let bytes_used = buf.len() - buffer.remaining();
    println!("Frame size:{} type:{} bytes_used:{}", frame.message_size, frame.type_id, bytes_used);

    stream.consume(bytes_used);

    let message_size = frame.message_size as usize;

    let mut buffer = if message_size > 0 {
        let buf = stream.fill_buf()?;
        if buf.len() < message_size {
            return Err(anyhow!("Buffer underrun; buf {}, message {}", buf.len(), message_size));
        }

        Bytes::copy_from_slice(&buf[..message_size])
    } else {
        Bytes::new()
    };

    println!("Message buffer {} - {:02x?}", buffer.len(), &buffer[..]);

    let message = ProtoMessage::decode(frame.type_id, &mut buffer)?;
    stream.consume(message_size);

    Ok(message)
}

pub fn write_message<S, M>(stream: &mut S, message: &M) -> Result<()>
    where S: Write, M: Message + MessageId
{
    let mut buffer = BytesMut::with_capacity(512);
    encode_message(message, &mut buffer)?;

    let buf = buffer.freeze();
    let sz = stream.write(&buf)?;
    println!("Write frame {} - {:02x?}", buf.len(), &buf[..]);

    Ok(())
}

pub fn write_encrypted_frame<S: Write>(stream: &mut S, payload: &[u8]) -> Result<()> {
    let mut buffer = BytesMut::new();

    buffer.put_u8(1);
    buffer.put_u16(payload.len() as u16);
    buffer.put(payload);

    let buf = buffer.freeze();
    stream.write_all(&buf)?;
    println!("Write frame {} - {:02x?}", buf.len(), &buf[..]);

    Ok(())
}

pub fn write_handshake_reject<S: Write>(stream: &mut S, reason: &str) -> Result<()> {
    let mut payload = vec![0x01];
    payload.extend_from_slice(reason.as_bytes());

    write_encrypted_frame(stream, payload.as_slice())
}

pub fn write_hello_frame<S: Write>(stream: &mut S,  node_name: &str, mac_addr: &str) -> Result<()> {
    let mut payload = vec![0x01];
    payload.extend_from_slice(node_name.as_bytes());
    payload.push(0);
    payload.extend_from_slice(mac_addr.as_bytes());
    payload.push(0);

    write_encrypted_frame(stream, payload.as_slice())
}

pub fn write_encrypted_message<S, M>(stream: &mut S, transport: &mut TransportState, message: &M) -> Result<()>
    where S: Write, M: Message + MessageId
{
    let mut message_buffer = BytesMut::with_capacity(512);
    encode_encrypted_message(message, &mut message_buffer)?;

    let buf = message_buffer.freeze();
    println!("Message for write {} - {:02x?}", buf.len(), &buf[..]);

    let mut buffer = vec![0u8; 512];
    let len = transport.write_message(&buf, &mut buffer)?;

    write_encrypted_frame(stream, &buffer[..len])?;

    Ok(())
}

pub fn read_encrypted_frame<R: BufRead>(stream: &mut R) -> Result<Bytes, ProtoError> {
    let buf = stream.fill_buf()?;
    if buf.len() == 0 {
        return Err(ProtoError::ReadZero);
    }

    let mut buffer = Bytes::copy_from_slice(buf);
    println!("Frame buffer {} - {:02x?}", buf.len(), buf);

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

pub fn read_message_encrypted<R: BufRead>(stream: &mut R, transport: &mut TransportState) -> Result<ProtoMessage> {
    let frame = read_encrypted_frame(stream)?;

    let mut buffer = vec![0u8; 512];
    let len = transport.read_message(&frame, &mut buffer)?;

    let mut buffer = Bytes::copy_from_slice(&buffer[..len]);

    let message_type = buffer.get_u16() as u64;
    let message_size = buffer.get_u16();

    ProtoMessage::decode(message_type, &mut buffer)
}

#[derive(thiserror::Error, Debug)]
pub enum ProtoError {
    #[error("Error reading from stream")]
    ReadError(#[from] std::io::Error),
    #[error("Read zero bytes expecting frame")]
    ReadZero,
    #[error("Expected first byte of frame to be {0}, found {1}")]
    InvalidIndicator(u8, u8),
    #[error("Buffer underrun; buf {0}, message {1}")]
    BufferUnderrun(usize, usize),
}
