/*
 * Nest UI - Home Assistant native thermostat interface
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

use std::{io::{BufReader, Read, Result as IoResult, Write}, thread, time::Duration};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use serial2::{SerialPort, Settings};

#[derive(thiserror::Error, Debug)]
pub enum BackplateError {
    #[error("IoError {0}")]
    IoError(#[from] std::io::Error),
    #[error("ChecksumMismatch")]
    ChecksumMismatch,
    #[error("InvalidAscii {0}")]
    InvalidAscii(#[from] std::string::FromUtf8Error)
}

pub type Result<T> = std::result::Result<T, BackplateError>;

pub fn open(path: &str) -> Result<()> {
    let mut port = SerialPort::open(path, |mut settings: Settings| {
        settings.set_raw();
        settings.set_baud_rate(115200)?;
        Ok(settings)
    })?;

    port.set_break(true)?;
    thread::sleep(Duration::from_millis(250));
    port.set_break(false)?;

    // Seems to help reduce (not eliminate) unexpected data in first few reads
    port.discard_buffers()?;

    let reset = Message {
        command_id: 0x00ff,
        payload: vec![]
    };

    println!("Send reset");
    reset.write(&mut port)?;

    let mut reader = MessageReader::new(&port)?;

    loop {
        if let Some(message) = reader.read_message()? {
            let message: BackplateMessage = message.try_into()?;
            match message {
                BackplateMessage::Text(s) => {
                    println!("Message (Text): {}", s);
                },
                BackplateMessage::Raw(m) => {
                    println!("Message {:?}", m);
                }
            }
        }

        thread::sleep(Duration::from_millis(250));
    }

    Ok(())
}

// https://github.com/mrhooray/crc-rs/issues/54
// CCITT is confusing because it's commonly misrepresented.
// You probably want CRC-16/KERMIT if init=0x0000 and CRC-16/IBM-3740 if init=0xffff.

// Through testing different algo's and crates using the reset example here:
// https://github.com/cuckoo-nest/wiki/blob/main/backplate/Protocol.md
// I landed on this (XMODEM)

fn crc_from_bytes(input: &[u8]) -> u16 {
    crc16::State::<crc16::XMODEM>::calculate(input)
}

fn crc_from_message(message: &Message) -> u16 {
    let mut buf = BytesMut::new();

    buf.put_u16_le(message.command_id);
    buf.put_u16_le(message.payload.len() as u16);
    buf.put(&message.payload[..]);

    crc_from_bytes(&buf.freeze())
}

#[derive(Debug)]
pub struct Message {
    command_id: u16,
    payload: Vec<u8>
}

impl Message {
    const PREAMBLE_WRITE: [u8; 3] = [0xd5, 0xaa, 0x96];
    const PREAMBLE_READ: [u8; 4] = [0xd5, 0xd5, 0xaa, 0x96];
    const MIN_RAW_LEN: usize = 10; // Header(4) + Cmd(2) + Len(2) + CRC(2)

    pub fn write<W: Write>(&self, stream: &mut W) -> IoResult<()> {
        let mut buf = BytesMut::new();

        buf.put_u16_le(self.command_id);
        buf.put_u16_le(self.payload.len() as u16);
        buf.put(&self.payload[..]);

        let crc_input = buf.clone().freeze();
        let checksum = crc_from_bytes(&crc_input);
        buf.put_u16_le(checksum);

        let mut message = vec![];
        message.extend_from_slice(&Self::PREAMBLE_WRITE);
        message.extend_from_slice(&buf.freeze());

        println!("Write {:x?}", message);
        stream.write(&message)?;

        Ok(())
    }

    pub fn parse(mut buffer: Bytes) -> Result<Option<(usize, Self)>> {
        // skip preamble; assume caller validated this
        buffer.advance(4);

        let command_id = buffer.get_u16_le();
        let data_len = buffer.get_u16_le() as usize;

        // check if buffer len includes payload and crc field
        if buffer.remaining() < data_len + 2 {
            return Ok(None);
        }

        let mut payload = vec![0; data_len];
        buffer.copy_to_slice(&mut payload);

        let message = Message {
            command_id,
            payload
        };

        let checksum = buffer.get_u16_le();
        let calc_checksum = crc_from_message(&message);
        if checksum != calc_checksum {
            return Err(BackplateError::ChecksumMismatch);
        }

        let read_len = Self::MIN_RAW_LEN + data_len;

        Ok(Some((read_len, message)))
    }
}

pub enum BackplateMessage {
    Text(String),
    Raw(Message)
}

impl TryFrom<Message> for BackplateMessage {
    type Error = BackplateError;

    fn try_from(value: Message) -> std::result::Result<Self, Self::Error> {
        let msg = match value {
            Message { command_id: 0x0001, payload } => {
                BackplateMessage::Text(String::from_utf8(payload)?)
            },
            msg => BackplateMessage::Raw(msg)
        };

        Ok(msg)
    }
}

struct MessageReader {
    reader: BufReader<SerialPort>,
    buffer: Vec<u8>
}

impl MessageReader {
    fn new(stream: &SerialPort) -> Result<Self> {
        Ok(Self {
            reader: BufReader::new(stream.try_clone()?),
            buffer: Vec::new()
        })
    }

    fn fill_buffer(&mut self) -> Result<usize> {
        let mut buf = vec![0; 512];
        let len = self.reader.read(&mut buf)?;
        self.buffer.put(&buf[..len]);
        println!("Read {:x?}", &buf[..len]);
        Ok(len)
    }

    fn read_message(&mut self) -> Result<Option<Message>> {
        // read from stream and append to self.buffer
        if self.buffer.len() < Message::MIN_RAW_LEN {
            self.fill_buffer()?;
        }

        println!("Buffered {:x?}", &self.buffer[..]);

        // search for preamble in buffer
        let preamble_pos = self.buffer
            .windows(4)
            .enumerate()
            .find(|(_, data)| *data == &Message::PREAMBLE_READ)
            .map(|(idx, _)| idx);

        if let Some(idx) = preamble_pos {
            // discard any data before preamble
            if idx > 0 {
                println!("MessageReader: discarding unexpected data {:x?}", &self.buffer[..idx]);
                self.buffer.drain(..idx);
            }

            let message_data = Bytes::from(self.buffer.clone());
            if let Some((len, message)) = Message::parse(message_data)? {
                println!("Parsed message, trimmed {} bytes from buffer", len);
                // remove parsed message data from buffer
                self.buffer.drain(..len);
                return Ok(Some(message))
            } else {
                // buffer doesn't contain full messages, read and try again
                self.fill_buffer()?;
                return self.read_message();
            }
        }

        Ok(None)
    }
}
