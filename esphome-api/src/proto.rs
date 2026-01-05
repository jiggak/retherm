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

use anyhow::Result;
use prost::{DecodeError, Message};

pub trait MessageId {
    const ID: u64;
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
    #[error("Error decoding protobuf message")]
    DecodeError(#[from] DecodeError),
    #[error("Error in noise decode or encode")]
    CodecError(#[from] snow::Error)
}

pub trait MessageReader {
    fn read(&mut self) -> Result<ProtoMessage, ProtoError>;
}

pub trait MessageWriter {
    fn write<M>(&mut self, message: &M) -> Result<()>
        where M: Message + MessageId;
}

pub struct ClimateFeature;

// Not currently exposed in api.proto
// https://github.com/esphome/esphome/blob/2025.12.4/esphome/components/climate/climate_mode.h#L104
impl ClimateFeature {
    /// Reporting current temperature is supported
    pub const SUPPORTS_CURRENT_TEMPERATURE: u32 = 1 << 0;
    /// Setting two target temperatures is supported (used in conjunction with CLIMATE_MODE_HEAT_COOL)
    pub const SUPPORTS_TWO_POINT_TARGET_TEMPERATURE: u32 = 1 << 1;
    /// Single-point mode is NOT supported (UI always displays two handles, setting 'target_temperature' is not supported)
    pub const REQUIRES_TWO_POINT_TARGET_TEMPERATURE: u32 = 1 << 2;
    /// Reporting current humidity is supported
    pub const SUPPORTS_CURRENT_HUMIDITY: u32 = 1 << 3;
    /// Setting a target humidity is supported
    pub const SUPPORTS_TARGET_HUMIDITY: u32 = 1 << 4;
    /// Reporting current climate action is supported
    pub const SUPPORTS_ACTION: u32 = 1 << 5;
}
