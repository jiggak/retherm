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

use anyhow::{Result, anyhow};
use embedded_graphics::{pixelcolor::{Bgr888, raw::ToBytes}, prelude::RgbColor};
use embedded_graphics_framebuf::FrameBuf;
use linuxfb::Framebuffer;

pub struct FramebufferDisplay {
    lfb: Framebuffer,
    pub buf: FrameBuf<Bgr888, [Bgr888; 320 * 320]>
}

impl FramebufferDisplay {
    pub fn new() -> Result<Self> {
        let lfb = Framebuffer::new("/dev/fb0")
            .or(Err(anyhow!("Error opening fb0")))?;

        let (width, height) = lfb.get_size();
        let (width, height) = (width as usize, height as usize);
        // let bpp = lfb.get_bytes_per_pixel() as usize;

        // let mut data = vec![Bgr888::BLACK; width * height];
        // FIXME vec as a backend doesn't compile, but do we care?
        // Nest framebuffer size is known and doesn't change.
        let data = [Bgr888::WHITE; 320 * 320];
        let buf = FrameBuf::new(data, width, height);

        Ok(Self { lfb, buf })
    }

    pub fn flush(&self) -> Result<()> {
        // Map the framebuffer into memory, so we can write to it:
        let mut fb_mem = self.lfb.map()
            .or(Err(anyhow!("Error mapping fb mem")))?;

        // FIXME is there a cleaner way to do this without building a new vector?
        let data: Vec<u8> = self.buf.data.iter()
            .flat_map(|p| {
                let b = p.to_be_bytes();
                // Adding a fourth byte per pixel... it appears to be unused
                // I've tried 0, 255, 10; I don't see a change in colour
                [b[0], b[1], b[2], 0]
            })
            .collect();

        // fb_mem.copy_from_slice(data.as_slice());
        // fb memory is twice as large as buffer source, double buffer?
        // maybe use set_virtual_size(320, 320) during init
        fb_mem[0..409600].copy_from_slice(data.as_slice());

        Ok(())
    }
}