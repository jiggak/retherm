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

use std::{
    cmp::min,
    fs,
    path::{Path, PathBuf},
    sync::mpsc::{Sender, channel},
    thread,
    time::Duration
};

use anyhow::Result;

#[derive(Clone)]
pub struct Backlight {
    device_dir: PathBuf,
    max_brightness: i32,
    default_brightness: i32
}

impl Backlight {
    pub fn new<P>(device_dir: P) -> Result<Self>
        where P: AsRef<Path>
    {
        let device_dir = device_dir.as_ref();

        // I would expect max brightness to be constant
        // It seems reasonable to read it just once, and hold on to it
        let file_path = device_dir.join("max_brightness");
        let max_brightness = fs::read_to_string(file_path)?
            .trim().parse()?;

        Ok(Self {
            device_dir: device_dir.to_path_buf(),
            max_brightness,
            default_brightness: 108
        })
    }

    pub fn set_brightness(&self, value: i32) -> Result<()> {
        let value = min(value, self.max_brightness);
        let file_path = self.device_dir.join("brightness");
        Ok(fs::write(file_path, value.to_string())?)
    }

    pub fn get_brightness(&self) -> Result<i32> {
        let file_path = self.device_dir.join("brightness");
        let brightness = fs::read_to_string(file_path)?
            .trim().parse()?;
        Ok(brightness)
    }

    pub fn start_timeout(&self, timeout_sec: u64) -> BacklightTimer {
        let timeout = Duration::from_secs(timeout_sec);
        BacklightTimer {
            backlight: self.clone(),
            timeout: timeout,
            timeout_reset: self.start_timeout_thread(timeout)
        }
    }

    fn start_timeout_thread(&self, timeout: Duration) -> Sender<()> {
        self.set_brightness(self.default_brightness).unwrap();

        let (sender, receiver) = channel();
        let backlight = self.clone();

        thread::spawn(move || {
            loop {
                // recv_timeout() returns Err when timeout reached
                // using sender of the channel resets the timeout
                if receiver.recv_timeout(timeout).is_err() {
                    break;
                }
            }

            backlight.set_brightness(0).unwrap();
        });

        sender
    }
}

pub struct BacklightTimer {
    backlight: Backlight,
    timeout: Duration,
    timeout_reset: Sender<()>
}

impl BacklightTimer {
    pub fn reset(&mut self) {
        // send() returns Err when timeout was previously reached
        if self.timeout_reset.send(()).is_err() {
            self.timeout_reset = self.backlight.start_timeout_thread(self.timeout);
        }
    }
}
