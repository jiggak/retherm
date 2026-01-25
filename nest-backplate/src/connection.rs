/*
 * ReTherm - Home Assistant native interface for Gen2 Nest thermostat
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

use std::{io::{BufReader, Read}};

use bytes::{BufMut, Bytes};
use log::trace;
use serial2::{SerialPort, Settings};

use crate::{BackplateCmd, BackplateResponse, Message, Result};

pub struct BackplateConnection {
    port: SerialPort,
    reader: MessageReader
}

impl BackplateConnection {
    pub fn send_command(&self, cmd: BackplateCmd) -> Result<()> {
        let message: Message = cmd.into();
        let message_data = message.to_bytes();
        trace!("Write {:x?}", &message_data[..]);
        self.port.write(&message_data)?;
        Ok(())
    }

    /// Read message from backplate. This method will not block forever. It will
    /// return a timeout error.
    pub fn read_message(&mut self) -> Result<BackplateResponse> {
        if let Some(message) = self.reader.read_message()? {
            Ok(message.try_into()?)
        } else {
            // There is more data to read (parial message) when read_message() returns `None`.
            self.read_message()
        }
    }

    pub fn open(path: &str) -> Result<Self> {
        let port = SerialPort::open(path, |mut settings: Settings| {
            settings.set_raw();
            settings.set_baud_rate(115200)?;
            Ok(settings)
        })?;

        // Nest Hacking wiki has tcsendbreak(fd, 1), Cuckoo Nest uses tcsendbreak(fd, 0)
        // Duration(1) = 1ms, Duration(0) = at least 250ms (< 500ms)
        // In my testing there doesn't appear to be any need for a delay at all.
        // In fact, when removing the `set_break` calls, everything still works
        // but I'm leaving them there since it doesn't hurt anything either.
        port.set_break(true)?;
        // thread::sleep(Duration::from_millis(250));
        port.set_break(false)?;

        // Seems to help reduce (not eliminate) unexpected data in first few reads
        port.discard_buffers()?;

        let reader = MessageReader::new(&port)?;

        let mut backplate = BackplateConnection { port, reader };

        backplate.send_command(BackplateCmd::Reset)?;

        let mut rcv_brk = false;
        let mut ack_payload: Option<Vec<u8>> = None;

        while !rcv_brk || ack_payload.is_none() {
            if let Some(raw_message) = backplate.reader.read_message()? {
                let raw_payload = raw_message.payload.clone();
                let message = raw_message.try_into()?;

                match message {
                    BackplateResponse::WirePowerPresence(_) => {
                        ack_payload = Some(raw_payload);
                    }
                    BackplateResponse::Text(s) if s == "BRK" => {
                        rcv_brk = true;
                    }
                    _ => { }
                }
            }
        }

        // This "Ack" command is required before messaging can be intitiated
        backplate.send_command(BackplateCmd::ResetAck(ack_payload.unwrap()))?;

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
        trace!("Read {:x?}", &buf[..len]);
        Ok(len)
    }

    fn read_message(&mut self) -> Result<Option<Message>> {
        // read from stream and append to self.buffer
        if self.buffer.len() < Message::MIN_RAW_LEN {
            self.fill_buffer()?;
        }

        trace!("Buffered {:x?}", &self.buffer[..]);

        // search for preamble in buffer
        let preamble_pos = self.buffer
            .windows(4)
            .enumerate()
            .find(|(_, data)| *data == &Message::PREAMBLE_READ)
            .map(|(idx, _)| idx);

        if let Some(idx) = preamble_pos {
            // discard any data before preamble
            if idx > 0 {
                trace!("Discarding unexpected data {:x?}", &self.buffer[..idx]);
                self.buffer.drain(..idx);
            }

            let message_data = Bytes::from(self.buffer.clone());
            if let Some((len, message)) = Message::parse(message_data)? {
                trace!("Parsed message, consumed {} bytes from buffer", len);
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
