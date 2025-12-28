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

use anyhow::Result;

mod backlight;
mod drawable;
mod event_pump;
mod input_events;
mod main_screen;
mod sound;
mod window;
#[cfg(feature = "device")]
mod window_fb;
#[cfg(feature = "simulate")]
mod window_sdl;

use crate::drawable::AppDrawable;
use crate::event_pump::Event;
use crate::main_screen::MainScreen;
use crate::window::AppWindow;

fn main() -> Result<()> {
    let mut window = get_window()?;

    let mut screen = MainScreen::new()?;

    'running: loop {
        screen.draw(window.draw_target())?;
        window.flush()?;

        let event = window.wait_event()?;
        if matches!(event, Event::Quit) {
            break 'running;
        }

        screen.handle_event(&event);
    }

    Ok(())
}

#[cfg(feature = "device")]
fn get_window() -> Result<impl AppWindow> {
    let window = crate::window_fb::FramebufferWindow::new()?;
    Ok(window)
}

#[cfg(feature = "simulate")]
fn get_window() -> Result<impl AppWindow> {
    let window = crate::window_sdl::SdlWindow::new()?;
    Ok(window)
}