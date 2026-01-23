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

mod connection;
pub use connection::*;
mod message;
pub use message::*;

#[derive(thiserror::Error, Debug)]
pub enum BackplateError {
    #[error("IoError {0}")]
    IoError(#[from] std::io::Error),
    #[error("ChecksumMismatch")]
    ChecksumMismatch,
    #[error("InvalidAscii {0}")]
    InvalidAscii(#[from] std::string::FromUtf8Error),
    #[error("{0}")]
    ParseError(String)
}

pub type Result<T> = std::result::Result<T, BackplateError>;
