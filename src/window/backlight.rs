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

use std::{cmp::min, fs, path::{Path, PathBuf}};

use anyhow::Result;

#[derive(Clone)]
pub struct Backlight {
    device: BacklightDirectory,
    max_brightness: u32,
    default_brightness: u32,
    current_brightness: u32
}

impl Backlight {
    pub fn load<P>(device_dir: P, default_brightness: u32) -> Result<Self>
        where P: AsRef<Path>
    {
        let device = BacklightDirectory::new(device_dir);

        // I would expect max brightness to be constant
        // It seems reasonable to read it just once, and hold on to it
        let max_brightness = device.read_value("max_brightness")?;
        let current_brightness = device.read_value("brightness")?;

        Ok(Self {
            device,
            max_brightness,
            default_brightness,
            current_brightness
        })
    }

    fn set_brightness(&mut self, value: u32) -> Result<()> {
        let value = min(value, self.max_brightness);

        if value != self.current_brightness {
            self.device.write_value("brightness", value)?;
            self.current_brightness = value;
        }

        Ok(())
    }

    pub fn turn_on(&mut self) -> Result<()> {
        self.set_brightness(self.default_brightness)
    }

    pub fn turn_off(&mut self) -> Result<()> {
        self.set_brightness(0)
    }
}

#[derive(Clone)]
struct BacklightDirectory {
    device_dir: PathBuf
}

impl BacklightDirectory {
    fn new<P: AsRef<Path>>(device_dir: P) -> Self {
        Self {
            device_dir: device_dir.as_ref().to_path_buf()
        }
    }

    fn read_value(&self, file_name: &str) -> Result<u32> {
        let file_path = self.device_dir.join(file_name);
        let value = fs::read_to_string(file_path)?
            .trim().parse()?;
        Ok(value)
    }

    fn write_value(&self, file_name: &str, value: u32) -> Result<()> {
        let file_path = self.device_dir.join(file_name);
        Ok(fs::write(file_path, value.to_string())?)
    }
}
