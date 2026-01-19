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
use embedded_graphics::{pixelcolor::Bgr888, prelude::*};
use embedded_graphics_framebuf::FrameBuf;
use linuxfb::Framebuffer;

use crate::{
    backlight::{Backlight, BacklightTimer}, drawable::AppDrawable,
    events::{Event, EventHandler}, sound::Sound
};

pub struct FramebufferWindow {
    fb_dev: Framebuffer,
    buffer: FrameBuf<Bgr888, [Bgr888; 320 * 320]>,
    backlight_timer: BacklightTimer,
    sounds: Sound
}

impl FramebufferWindow {
    pub fn new() -> Result<Self> {
        let mut fb_dev = Framebuffer::new("/dev/fb0")
            .or(Err(anyhow!("Error opening fb0")))?;

        // sometimes the offset will be (0, 320) after opening fb0
        // causing nothing to appear on screen
        fb_dev.set_offset(0, 0)
            .or(Err(anyhow!("Error changing offset of fb0")))?;

        let (width, height) = fb_dev.get_size();
        let (width, height) = (width as usize, height as usize);
        // let bpp = fb.get_bytes_per_pixel() as usize;

        // let mut data = vec![Bgr888::BLACK; width * height];
        // FIXME vec as a backend doesn't compile, but do we care?
        // Nest framebuffer size is known and doesn't change.
        let data = [Bgr888::WHITE; 320 * 320];
        let buffer = FrameBuf::new(data, width, height);

        let backlight = Backlight::new("/sys/class/backlight/3-0036")?;
        let backlight_timer = backlight.start_timeout(15);

        let sounds = Sound::new()?;

        Ok(Self { fb_dev, buffer, backlight_timer, sounds })
    }

    fn flush(&self) -> Result<()> {
        // Map the framebuffer into memory, so we can write to it:
        let mut fb_mem = self.fb_dev.map()
            .or(Err(anyhow!("Error mapping fb0 mem")))?;

        // FIXME If I can somehow efficiently get the buffer data as u8 slice
        // then we can use a memcpy with fb_mem.copy_from_slice()
        for (i, p) in self.buffer.data.iter().enumerate() {
            let offset = i*4;
            fb_mem[offset] = p.b();
            fb_mem[offset+1] = p.g();
            fb_mem[offset+2] = p.r();
            // Fourth byte appears to be unused.
            // I've tried 0, 255, 10; I don't see a change in colour
        }

        Ok(())
    }

    pub fn draw_screen(&mut self, screen: &dyn AppDrawable) -> Result<()> {
        screen.draw(&mut self.buffer)?;
        self.flush()?;
        Ok(())
    }
}

impl EventHandler for FramebufferWindow {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        if event.is_wakeup_event() {
            self.backlight_timer.reset();
        }

        if matches!(event, Event::Dial(_)) {
            self.sounds.click()?;
        }

        Ok(())
    }
}
