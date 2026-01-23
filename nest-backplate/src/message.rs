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

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::{BackplateError, Result};

#[derive(Debug)]
pub struct Message {
    pub command_id: u16,
    pub payload: Vec<u8>
}

impl Message {
    const PREAMBLE_WRITE: [u8; 3] = [0xd5, 0xaa, 0x96];
    pub(crate) const PREAMBLE_READ: [u8; 4] = [0xd5, 0xd5, 0xaa, 0x96];

    /// Preamble(4) + Cmd(2) + Len(2) + CRC(2)
    pub(crate) const MIN_RAW_LEN: usize = 10;

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
            return Err(BackplateError::ChecksumMismatch {
                recv: checksum,
                calc: calc_checksum
            });
        }

        let read_len = Self::MIN_RAW_LEN + data_len;

        Ok(Some((read_len, message)))
    }
}

#[derive(Debug)]
pub enum BackplateCmd {
    Reset,
    ResetAck(Vec<u8>),
    GetTfeVersion,
    GetTfeBuildInfo,
    GetBackplateModelAndBslId,
    SetPowerStealMode,
    StatusRequest,
    SwitchWire(Wire, bool)
}

impl From<BackplateCmd> for Message {
    fn from(value: BackplateCmd) -> Self {
        match value {
            BackplateCmd::StatusRequest => {
                Message::command(0x0083)
            }
            BackplateCmd::SwitchWire(wire, enabled) => {
                let wire = wire.to_byte();
                let enabled = if enabled { 0x01 } else { 0x00 };
                Message::with_payload(0x0082, vec![wire, enabled])
            }
            BackplateCmd::ResetAck(data) => {
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
            BackplateCmd::Reset => {
                Message::command(0x00ff)
            }
        }
    }
}

#[derive(Debug)]
pub enum BackplateResponse {
    Text(String),
    WirePowerPresence(BackplateWires<bool>),
    WirePluggedPresence(BackplateWires<bool>),
    WireSwitched(Wire, bool),
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
                let (temp, hum) = match payload.as_chunks::<2>() {
                    ([t, h, ..], _) => {
                        Ok((u16::from_le_bytes(*t), u16::from_le_bytes(*h)))
                    },
                    _ => Err(BackplateError::PayloadLength {
                        id: 0x0002, expected: 4, found: payload.len()
                    })
                }?;

                BackplateResponse::Climate {
                    temperature: temp as f32 / 100.0,
                    humidity: hum as f32 / 10.0
                }
            }
            Message { command_id: 0x0004, payload } => {
                if payload.len() < 12 {
                    return Err(BackplateError::PayloadLength {
                        id: 0x0004, expected: 12, found: payload.len()
                    });
                }

                // W1, Y1, G, OB, W2, ?0, ?0, Y2, C, RC, RH, *, ?0
                // Mapping from https://wiki.exploitee.rs/index.php/Nest_Hacking
                // I was able to confirm Rc and Rh by testing with 9V batt
                // connected to R[c,h] and C. Other mapping is unconfirmed.
                let wires = BackplateWires {
                    w1: payload[0] == 1,
                    y1: payload[1] == 1,
                    g: payload[2] == 1,
                    ob: payload[3] == 1,
                    w2: payload[4] == 1,
                    y2: payload[7] == 1,
                    c: payload[8] == 1,
                    rc: payload[9] == 1,
                    rh: payload[10] == 1,
                    star: payload[11] == 1
                };

                BackplateResponse::WirePowerPresence(wires)
            }
            Message { command_id: 0x0006, payload } => {
                let (wire, enabled) = match payload.as_slice() {
                    [b0, b1, ..] => {
                        Ok((Wire::try_from_byte(*b0)?, *b1 == 1))
                    },
                    _ => Err(BackplateError::PayloadLength {
                        id: 0x0006, expected: 2, found: payload.len()
                    })
                }?;

                BackplateResponse::WireSwitched(wire, enabled)
            }
            Message { command_id: 0x0007, payload } => {
                let proximity = match payload.as_slice() {
                    [b0, b1, ..] => {
                        Ok(u16::from_le_bytes([*b0, *b1]))
                    },
                    _ => Err(BackplateError::PayloadLength {
                        id: 0x0007, expected: 2, found: payload.len()
                    })
                }?;

                BackplateResponse::ProximitySensor(proximity)
            }
            Message { command_id: 0x0009, payload } => {
                if payload.len() < 12 {
                    return Err(BackplateError::PayloadLength {
                        id: 0x0009, expected: 12, found: payload.len()
                    });
                }

                // Mapping observed by testing each wire on Model 02A backplate
                let wires = BackplateWires {
                    w1: payload[0] == 1,
                    y1: payload[1] == 1,
                    c: payload[2] == 1,
                    rc: payload[3] == 1,
                    rh: payload[4] == 1,
                    g: payload[5] == 1,
                    ob: payload[6] == 1,
                    w2: payload[7] == 1,
                    y2: payload[9] == 1,
                    star: payload[11] == 1
                };
                BackplateResponse::WirePluggedPresence(wires)
            }
            Message { command_id: 0x000a, payload } => {
                // 4 byte payload, but only the first two bytes seem to change
                // with light shining at device
                let lux = match payload.as_slice() {
                    [b0, b1, ..] => {
                        Ok(u16::from_le_bytes([*b0, *b1]))
                    },
                    _ => Err(BackplateError::PayloadLength {
                        id: 0x000a, expected: 2, found: payload.len()
                    })
                }?;

                BackplateResponse::AmbientLightSensor(lux)
            }
            Message { command_id: 0x000b, payload } => {
                if payload.len() < 14 {
                    return Err(BackplateError::PayloadLength {
                        id: 0x0009, expected: 12, found: payload.len()
                    });
                }

                let charging = payload[1] & 0x40 != 0;
                let vin = u16::from_le_bytes([payload[8], payload[9]]);
                let vop = u16::from_le_bytes([payload[10], payload[11]]);
                let vbat = u16::from_le_bytes([payload[12], payload[13]]);

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
            msg => BackplateResponse::Raw(msg)
        };

        Ok(result)
    }
}

#[derive(Debug)]
pub struct BackplateWires<T> {
    pub y1: T,
    pub y2: T,
    pub g: T,
    pub ob: T,
    pub rc: T,

    pub w1: T,
    pub w2: T,
    pub c: T,
    pub star: T,
    pub rh: T
}

#[derive(Debug)]
pub enum Wire {
    W1, Y1, G, OB, W2, Y2, Star
}

impl Wire {
    fn to_byte(&self) -> u8 {
        match self {
            Self::W1 => 0x00,
            Self::Y1 => 0x01,
            Self::G => 0x02,
            Self::OB => 0x03,
            Self::W2 => 0x04,
            Self::Y2 => 0x07,
            Self::Star => 0x0b
        }
    }

    fn try_from_byte(id: u8) -> Result<Self> {
        match id {
            0x00 => Ok(Self::W1),
            0x01 => Ok(Self::Y1),
            0x02 => Ok(Self::G),
            0x03 => Ok(Self::OB),
            0x04 => Ok(Self::W2),
            0x07 => Ok(Self::Y2),
            0x0b => Ok(Self::Star),
            _ => Err(BackplateError::InvalidWireId(id))
        }
    }
}

// https://github.com/mrhooray/crc-rs/issues/54
// > CCITT is confusing because it's commonly misrepresented.
// > You probably want CRC-16/KERMIT if init=0x0000 and CRC-16/IBM-3740 if init=0xffff.

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
