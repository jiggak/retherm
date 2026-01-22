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

use std::{io::{BufReader, Read}, thread, time::Duration};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use serial2::{SerialPort, Settings};

// Implementation based on information gathered from:
// https://github.com/cuckoo-nest/wiki/blob/main/backplate/Protocol.md
// https://wiki.exploitee.rs/index.php/Nest_Hacking
// Other implementations for reference:
// https://github.com/cuckoo-nest/cuckoo_hello/
// https://github.com/cuckoo-nest/cuckoo_nest/

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

pub struct BackplateConnection {
    port: SerialPort,
    reader: MessageReader
}

impl BackplateConnection {
    pub fn send_command(&self, cmd: BackplateCmd) -> Result<()> {
        let message: Message = cmd.into();
        let message_data = message.to_bytes();
        // println!("Write {:x?}", &message_data[..]);
        self.port.write(&message_data)?;
        Ok(())
    }

    pub fn read_message(&mut self) -> Result<Option<BackplateResponse>> {
        if let Some(message) = self.reader.read_message()? {
            Ok(Some(message.try_into()?))
        } else {
            Ok(None)
        }
    }

    pub fn open(path: &str) -> Result<Self> {
        let port = SerialPort::open(path, |mut settings: Settings| {
            settings.set_raw();
            settings.set_baud_rate(115200)?;
            Ok(settings)
        })?;

        port.set_break(true)?;
        thread::sleep(Duration::from_millis(250));
        port.set_break(false)?;

        // Seems to help reduce (not eliminate) unexpected data in first few reads
        port.discard_buffers()?;

        let reader = MessageReader::new(&port)?;

        let mut backplate = BackplateConnection { port, reader };

        println!("Send reset");
        backplate.send_command(BackplateCmd::Reset)?;

        let mut rcv_brk = false;
        let mut fet_payload: Option<Vec<u8>> = None;

        while !rcv_brk || fet_payload.is_none() {
            if let Some(message) = backplate.read_message()? {
                println!("{:?}", message);
                match message {
                    BackplateResponse::FetPresence(data) => {
                        fet_payload = Some(data);
                    }
                    BackplateResponse::Text(s) if s == "BRK" => {
                        rcv_brk = true;
                    }
                    _ => { }
                }
            }

            thread::sleep(Duration::from_millis(250));
        }

        println!("Sending FET ACK");
        backplate.send_command(BackplateCmd::FetPresenceAck(fet_payload.unwrap()))?;

        // The Cuckoo Nest implementation sends a series of commands to fetch
        // info (e.g. GetTfeVersion) but this doesn't seem to be necessary.

        // What does SetPowerStealMode do?
        // I've tested with/without this command and I don't notice any difference.
        // My assumption is this command has some effect on charging.
        // With a 9V batt connected to Rh/C the voltage levels in state messages
        // show voltage readings and a bit is flipped seemingly to indicate
        // charging state (byte 1, bit 6 [zero based]).
        // These values do not change with/without this command.
        backplate.send_command(BackplateCmd::SetPowerStealMode)?;

        Ok(backplate)
    }
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

    /// Preamble(4) + Cmd(2) + Len(2) + CRC(2)
    const MIN_RAW_LEN: usize = 10;

    pub fn command(command_id: u16) -> Self {
        Self { command_id, payload: Vec::new() }
    }

    pub fn with_payload(command_id: u16, payload: Vec<u8>) -> Self {
        Self { command_id, payload }
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buf = BytesMut::new();

        buf.put(&Self::PREAMBLE_WRITE[..]);
        buf.put_u16_le(self.command_id);
        buf.put_u16_le(self.payload.len() as u16);
        buf.put(&self.payload[..]);

        // calc crc of ID, Len, Payload (no preamble)
        let checksum = crc_from_bytes(&buf[3..]);
        buf.put_u16_le(checksum);

        buf.freeze()
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

#[derive(Debug)]
pub enum BackplateCmd {
    Reset,
    FetPresenceAck(Vec<u8>),
    GetTfeVersion,
    GetTfeBuildInfo,
    GetBackplateModelAndBslId,
    SetPowerStealMode,
    StatusRequest
}

impl From<BackplateCmd> for Message {
    fn from(value: BackplateCmd) -> Self {
        match value {
            BackplateCmd::Reset => {
                Message::command(0x00ff)
            }
            BackplateCmd::FetPresenceAck(data) => {
                Message::with_payload(0x008f, data)
            }
            BackplateCmd::GetTfeVersion => {
                Message::command(0x0098)
            }
            BackplateCmd::GetTfeBuildInfo => {
                Message::command(0x0099)
            }
            BackplateCmd::GetBackplateModelAndBslId => {
                Message::command(0x009d)
            }
            BackplateCmd::SetPowerStealMode => {
                Message::with_payload(0x00c0, vec![0x00, 0x00, 0x00, 0x00])
            }
            BackplateCmd::StatusRequest => {
                Message::command(0x0083)
            }
        }
    }
}

#[derive(Debug)]
pub enum BackplateResponse {
    Text(String),
    FetPresence(Vec<u8>),
    TfeVersion(String),
    TfeBuildInfo(String),
    BackplateModelAndBslId(Vec<u8>),
    ProximitySensor(u16),
    AmbientLightSensor(u16),
    BackplateState {
        charging: bool,
        volts_in: f32,
        volts_op: f32,
        volts_bat: f32
    },
    Climate {
        temperature: f32,
        humidity: f32
    },
    Raw(Message)
}

impl TryFrom<Message> for BackplateResponse {
    type Error = BackplateError;

    fn try_from(value: Message) -> Result<Self> {
        let result = match value {
            Message { command_id: 0x0001, payload } => {
                BackplateResponse::Text(String::from_utf8(payload)?)
            }
            Message { command_id: 0x0002, payload } => {
                let temp = u16::from_le_bytes(payload[..2].try_into().unwrap());
                let humidity = u16::from_le_bytes(payload[2..4].try_into().unwrap());
                BackplateResponse::Climate {
                    temperature: temp as f32 / 100.0,
                    humidity: humidity as f32 / 10.0
                }
            }
            Message { command_id: 0x0004, payload } => {
                BackplateResponse::FetPresence(payload)
            }
            Message { command_id: 0x0007, payload } => {
                let proximity = u16::from_le_bytes(payload.try_into().unwrap());
                BackplateResponse::ProximitySensor(proximity)
            }
            Message { command_id: 0x000a, payload } => {
                // 4 byte payload, but only the first two bytes seem to change
                // with light shining at device
                let lux = u16::from_le_bytes(payload[..2].try_into().unwrap());
                BackplateResponse::AmbientLightSensor(lux)
            }
            Message { command_id: 0x000b, payload } => {
                let charging = payload[1] & 0x40 != 0;
                let vin = u16::from_le_bytes(payload[8..10].try_into().unwrap());
                let vop = u16::from_le_bytes(payload[10..12].try_into().unwrap());
                let vbat = u16::from_le_bytes(payload[12..14].try_into().unwrap());
                BackplateResponse::BackplateState {
                    charging,
                    volts_in: vin as f32 / 100.0,
                    volts_op: vop as f32 / 1000.0,
                    volts_bat: vbat as f32 / 1000.0
                }
            }
            Message { command_id: 0x0018, payload } => {
                BackplateResponse::TfeVersion(String::from_utf8(payload)?)
            }
            Message { command_id: 0x0019, payload } => {
                BackplateResponse::TfeBuildInfo(String::from_utf8(payload)?)
            }
            Message { command_id: 0x001d, payload } => {
                BackplateResponse::BackplateModelAndBslId(payload)
            }
            val => BackplateResponse::Raw(val)
        };

        Ok(result)
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
        // println!("Read {:x?}", &buf[..len]);
        Ok(len)
    }

    fn read_message(&mut self) -> Result<Option<Message>> {
        // read from stream and append to self.buffer
        if self.buffer.len() < Message::MIN_RAW_LEN {
            self.fill_buffer()?;
        }

        // println!("Buffered {:x?}", &self.buffer[..]);

        // search for preamble in buffer
        let preamble_pos = self.buffer
            .windows(4)
            .enumerate()
            .find(|(_, data)| *data == &Message::PREAMBLE_READ)
            .map(|(idx, _)| idx);

        if let Some(idx) = preamble_pos {
            // discard any data before preamble
            if idx > 0 {
                // println!("MessageReader: discarding unexpected data {:x?}", &self.buffer[..idx]);
                self.buffer.drain(..idx);
            }

            let message_data = Bytes::from(self.buffer.clone());
            if let Some((len, message)) = Message::parse(message_data)? {
                // println!("Parsed message, consumed {} bytes from buffer", len);
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
