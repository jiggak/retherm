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

use std::{fs, io::Result, path::{Path, PathBuf}};

pub struct Backlight {
    device_dir: PathBuf
}

impl Backlight {
    pub fn new<P>(device_dir: P) -> Self
        where P: AsRef<Path>
    {
        Self {
            device_dir: device_dir.as_ref().to_path_buf()
        }
    }

    fn device_file(&self, file_name: &str) -> PathBuf {
        self.device_dir.join(file_name)
    }

    pub fn set_brightness(&self, value: i32) -> Result<()> {
        fs::write(self.device_file("brightness"), value.to_string())
    }
}