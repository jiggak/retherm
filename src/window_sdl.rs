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

use std::convert::Infallible;

use anyhow::{Result, anyhow};
use embedded_graphics::{pixelcolor::Bgr888, prelude::*};
use embedded_graphics_framebuf::FrameBuf;
use sdl2::{
    EventPump,
    event::Event as SdlEvent,
    pixels::PixelFormatEnum,
    render::Canvas,
    video::Window
};

use crate::{event_pump::Event, window::AppWindow};

pub struct SdlWindow {
    window_canvas: Canvas<Window>,
    buffer: FrameBuf<Bgr888, [Bgr888; 320 * 320]>,
    event_pump: EventPump
}

impl SdlWindow {
    pub fn new() -> Result<Self> {
        let sdl_context = sdl2::init()
            .map_err(|e| anyhow!(e))?;

        let window = sdl_context.video()
            .map_err(|e| anyhow!(e))?
            .window("Nest App", 320, 320)
            .position_centered()
            .build()?;

        let window_canvas = window.into_canvas()
            .build()
            .map_err(|e| anyhow!(e))?;

        let data = [Bgr888::WHITE; 320 * 320];
        let buffer = FrameBuf::new(data, 320, 320);

        let event_pump = sdl_context.event_pump()
            .map_err(|e| anyhow!(e))?;

        Ok(
            Self { window_canvas, buffer, event_pump }
        )
    }
}

impl AppWindow for SdlWindow {
    fn draw_target(&mut self) -> &mut impl DrawTarget<Color = Bgr888, Error = Infallible> {
        &mut self.buffer
    }

    fn flush(&mut self) -> Result<()> {
        let texture_creator = self.window_canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::BGR888, 320, 320)
            .map_err(|e| anyhow!(e))?;

        texture.with_lock(None, |dest, _| {
            for (i, p) in self.buffer.data.iter().enumerate() {
                let offset = i*4;
                dest[offset] = p.b();
                dest[offset + 1] = p.g();
                dest[offset + 2] = p.r();
            }
        }).map_err(|e| anyhow!(e))?;

        self.window_canvas.clear();
        self.window_canvas.copy(&texture, None, None)
            .map_err(|e| anyhow!(e))?;
        self.window_canvas.present();

        Ok(())
    }

    fn wait_event(&mut self) -> Result<Event> {
        match self.event_pump.wait_event() {
            SdlEvent::Quit { .. } => Ok(Event::Quit),
            SdlEvent::MouseButtonDown { .. } => Ok(Event::ButtonDown),
            SdlEvent::MouseWheel { y, .. } => Ok(Event::Dial(y)),
            _ => self.wait_event()
        }
    }
}
