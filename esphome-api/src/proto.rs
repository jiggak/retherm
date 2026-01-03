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

pub fn read_message<R>(stream: &mut R) -> Result<ProtoMessage>
    where R: BufRead
{
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
    println!("Write {} bytes", sz);

    Ok(())
}
