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

mod backlight;
mod drawable;
mod events;
// mod home_assistant;
mod input_events;
mod main_screen;
mod sound;
mod window;
#[cfg(feature = "device")]
mod window_fb;
#[cfg(feature = "simulate")]
mod window_sdl;

use anyhow::Result;
// use esphome_api::server::EncryptedStreamProvider;

use crate::drawable::AppDrawable;
use crate::events::{Event, EventHandler, EventSource};
// use crate::home_assistant::HomeAssistant;
use crate::main_screen::MainScreen;
use crate::window::AppWindow;

fn main() -> Result<()> {
    let mut event_source = get_event_source()?;
    let mut window = get_window()?;
    let mut screen = MainScreen::new(&event_source)?;

    start_threads(&event_source)?;

    // let stream_factory = EncryptedStreamProvider::new(
    //     "jfD5V1SMKAPXNC8+d6BvE1EGBHJbyw2dSc0Q+ymNMhU=",
    //     "test-thermostat",
    //     "01:02:03:04:05:06"
    // )?;

    // let mut home_assistant = HomeAssistant::new(event_source.event_sender());
    // home_assistant.start_listener("0.0.0.0:6053", stream_factory);

    // let mut handlers: Vec<&mut dyn EventHandler> = vec![
    //     &mut window, &mut screen
    // ];

    'running: loop {
        screen.draw(window.draw_target())?;
        window.flush()?;

        let event = event_source.wait_event()?;
        if matches!(event, Event::Quit) {
            break 'running;
        }

        window.handle_event(&event)?;
        screen.handle_event(&event)?;
        // home_assistant.handle_event(&event)?;

        // for handler in handlers.iter_mut() {
        //     handler.handle_event(&event);
        // }
    }

    Ok(())
}

#[cfg(feature = "device")]
fn get_window() -> Result<crate::window_fb::FramebufferWindow> {
    crate::window_fb::FramebufferWindow::new()
}

#[cfg(feature = "device")]
fn get_event_source() -> Result<impl EventSource> {
    Ok(crate::events::DefaultEventSource::new())
}

#[cfg(feature = "device")]
fn start_threads<E: EventSource>(events: &E) -> Result<()> {
    crate::input_events::start_button_events(events.event_sender())?;
    crate::input_events::start_dial_events(events.event_sender())?;
    Ok(())
}

#[cfg(feature = "simulate")]
fn get_window() -> Result<crate::window_sdl::SdlWindow> {
    crate::window_sdl::SdlWindow::new()
}

#[cfg(feature = "simulate")]
fn get_event_source() -> Result<impl EventSource> {
    crate::window_sdl::SdlEventSource::new()
}

#[cfg(feature = "simulate")]
fn start_threads<E: EventSource>(_events: &E) -> Result<()> {
    Ok(())
}
