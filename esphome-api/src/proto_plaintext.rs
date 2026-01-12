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

use prost::{bytes::{Buf, BufMut, Bytes, BytesMut}, encoding::{decode_varint, encode_varint}};

use crate::proto::{MessageReader, MessageStream, MessageWriter, ProtoError, ProtoMessage};

pub struct PlaintextMessageStream {
    reader: BufReader<TcpStream>
}

impl PlaintextMessageStream {
    pub fn new(reader: BufReader<TcpStream>) -> Self {
        Self { reader }
    }
}

impl MessageStream for PlaintextMessageStream {
    fn clone(&self) -> Self {
        let stream = self.reader.get_ref().try_clone().unwrap();
        PlaintextMessageStream { reader: BufReader::new(stream) }
    }
}

impl MessageReader for PlaintextMessageStream {
    fn read(&mut self) -> Result<ProtoMessage, ProtoError> {
        let buf = self.reader.fill_buf()?;
        if buf.len() == 0 {
            return Err(ProtoError::ReadZero);
        }

        let mut buffer = Bytes::copy_from_slice(buf);

        let byte_zero = buffer.get_u8();
        if byte_zero != 0 {
            return Err(ProtoError::InvalidIndicator(0, byte_zero));
        }

        let message_size = decode_varint(&mut buffer)? as usize;
        let message_type = decode_varint(&mut buffer)?;

        let bytes_used = buf.len() - buffer.remaining();
        self.reader.consume(bytes_used);

        let mut buffer = if message_size > 0 {
            let buf = self.reader.fill_buf()?;
            if buf.len() < message_size {
                return Err(ProtoError::BufferUnderrun(buf.len(), message_size));
            }

            Bytes::copy_from_slice(&buf[..message_size])
        } else {
            Bytes::new()
        };

        let message = ProtoMessage::decode(message_type, &mut buffer)?;
        self.reader.consume(message_size);

        Ok(message)
    }
}

impl MessageWriter for PlaintextMessageStream {
    fn write(&mut self, message: &ProtoMessage) -> Result<(), ProtoError> {
        let mut buffer = BytesMut::with_capacity(512);
        encode_message(message, &mut buffer)?;

        let buf = buffer.freeze();
        self.reader.get_ref().write_all(&buf)?;

        Ok(())
    }
}

fn encode_message<B: BufMut>(message: &ProtoMessage, buffer: &mut B) -> Result<(), ProtoError> {
    let message_id = message.message_id();
    let message_len = message.encoded_len();

    buffer.put_u8(0u8);
    encode_varint(message_len as u64, buffer);
    encode_varint(message_id, buffer);
    message.encode(buffer)?;

    Ok(())
}
