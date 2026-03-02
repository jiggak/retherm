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

use std::sync::Arc;

use anyhow::{Result, anyhow};
use embedded_graphics::{pixelcolor::Bgr888, prelude::*};
use embedded_graphics_framebuf::FrameBuf;
use sdl2::{
    EventPump, event::{Event as SdlEvent, EventSender as SdlEventSender},
    keyboard::Keycode, pixels::PixelFormatEnum, render::Canvas, video::Window
};

use crate::{drawable::AppDrawable, events::{Event, EventHandler, EventSender, EventSource}};

pub struct SdlWindow {
    window_canvas: Canvas<Window>,
    buffer: FrameBuf<Bgr888, [Bgr888; 320 * 320]>
}

impl SdlWindow {
    pub fn new() -> Result<Self> {
        let sdl_context = sdl2::init()
            .map_err(|e| anyhow!(e))?;

        let window = sdl_context.video()
            .map_err(|e| anyhow!(e))?
            .window("ReTherm", 320, 320)
            .position_centered()
            .build()?;

        let window_canvas = window.into_canvas()
            .build()
            .map_err(|e| anyhow!(e))?;

        let data = [Bgr888::WHITE; 320 * 320];
        let buffer = FrameBuf::new(data, 320, 320);

        Ok(
            Self { window_canvas, buffer }
        )
    }

    fn flush(&mut self) -> Result<()> {
        let texture_creator = self.window_canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(PixelFormatEnum::BGR888, 320, 320)
            .map_err(|e| anyhow!(e))?;

        texture.with_lock(None, |dest, _| {
            for (i, p) in self.buffer.data.iter().enumerate() {
                let offset = i*4;
                dest[offset] = p.r();
                dest[offset + 1] = p.g();
                dest[offset + 2] = p.b();
            }
        }).map_err(|e| anyhow!(e))?;

        self.window_canvas.clear();
        self.window_canvas.copy(&texture, None, None)
            .map_err(|e| anyhow!(e))?;
        self.window_canvas.present();

        Ok(())
    }

    pub fn draw_screen(&mut self, screen: &dyn AppDrawable) -> Result<()> {
        screen.draw(&mut self.buffer)?;
        self.flush()?;
        Ok(())
    }
}

impl EventHandler for SdlWindow {
    fn handle_event(&mut self, _event: &Event) -> Result<()> {
        Ok(())
    }
}

pub struct SdlEventSource {
    event_pump: EventPump,
    event_sender: SdlEventSenderHandle,
    current_temp: f32
}

impl SdlEventSource {
    pub fn new() -> Result<Self> {
        let sdl_context = sdl2::init()
            .map_err(|e| anyhow!(e))?;

        let event_pump = sdl_context.event_pump()
            .map_err(|e| anyhow!(e))?;

        let sdl_events = sdl_context.event()
            .map_err(|e| anyhow!(e))?;

        sdl_events.register_custom_event::<Event>()
            .map_err(|e| anyhow!(e))?;

        let event_sender = SdlEventSenderHandle::new(sdl_events.event_sender());

        Ok(Self {
            event_pump,
            event_sender,
            current_temp: 20.0
        })
    }

    fn map_sdl_event(&mut self, event: SdlEvent) -> Option<Event> {
        match event {
            SdlEvent::Quit { .. } =>
                Some(Event::Quit),
            SdlEvent::MouseButtonDown { .. } =>
                Some(Event::ButtonDown),
            SdlEvent::MouseWheel { y, .. } if y != 0 =>
                Some(Event::Dial(y * 10)),
            SdlEvent::KeyDown { keycode, .. } if keycode == Some(Keycode::Up) =>
                Some(Event::Dial(20)),
            SdlEvent::KeyDown { keycode, .. } if keycode == Some(Keycode::Down) =>
                Some(Event::Dial(-20)),
            SdlEvent::KeyDown { keycode, .. } if keycode == Some(Keycode::P) =>
                Some(Event::ProximityNear),
            SdlEvent::KeyDown { keycode, .. } if keycode == Some(Keycode::LEFTBRACKET) => {
                self.current_temp = self.current_temp - 0.1;
                Some(Event::SetCurrentTemp(self.current_temp))
            }
            SdlEvent::KeyDown { keycode, .. } if keycode == Some(Keycode::RIGHTBRACKET) => {
                self.current_temp = self.current_temp + 0.1;
                Some(Event::SetCurrentTemp(self.current_temp))
            }
            sdl_event => {
                if sdl_event.is_user_event() {
                    Some(sdl_event.as_user_event_type::<Event>().unwrap())
                } else {
                    None
                }
            }
        }
    }
}

impl EventSource<SdlEventSenderHandle> for SdlEventSource {
    fn wait_event(&mut self) -> Result<Event> {
        let event = self.event_pump.wait_event();
        if let Some(event) = self.map_sdl_event(event) {
            Ok(event)
        } else {
            // Unhandled event: wait again
            self.wait_event()
        }
    }

    fn poll_event(&mut self) -> Result<Option<Event>> {
        if let Some(event) = self.event_pump.poll_event() {
            if let Some(event) = self.map_sdl_event(event) {
                Ok(Some(event))
            } else {
                // Unhandled event: poll again
                self.poll_event()
            }
        } else {
            Ok(None)
        }
    }

    fn event_sender(&self) -> SdlEventSenderHandle {
        self.event_sender.clone()
    }
}

#[derive(Clone)]
pub struct SdlEventSenderHandle {
    inner: Arc<SdlEventSender>,
}

impl SdlEventSenderHandle {
    fn new(sender: SdlEventSender) -> Self {
        Self { inner: Arc::new(sender) }
    }
}

impl EventSender for SdlEventSenderHandle {
    fn send_event(&self, event: Event) -> Result<()> {
        self.inner.push_custom_event(event)
            .map_err(|e| anyhow!(e))
    }
}
