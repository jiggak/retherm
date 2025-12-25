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
mod display;
mod display_framebuffer;
mod drawable;
mod event_pump;
mod input_events;
mod main_screen;
mod sound;

use crate::backlight::Backlight;
use crate::display::Display;
use crate::display_framebuffer::FramebufferDisplay;
use crate::drawable::AppDrawable;
use crate::event_pump::{Event, EventPump};
use crate::input_events::InputEvents;
use crate::main_screen::MainScreen;
use crate::sound::Sound;

fn main() -> Result<()> {
    let mut display = get_display()?;

    let mut screen = MainScreen::new();

    let backlight = Backlight::new("/sys/class/backlight/3-0036")?;
    let mut backlight_timer = backlight.start_timeout(15);

    let event_pump = EventPump::new();
    let input_events = InputEvents::new()?;
    input_events.start_polling(event_pump.sender.clone());

    let mut sounds = Sound::new()?;

    loop {
        screen.draw(display.draw_target())?;
        display.flush()?;

        let event = event_pump.wait_event()?;

        if event.is_wakeup_event() {
            backlight_timer.reset();
        }

        if matches!(event, Event::Dial(..)) {
            sounds.click()?;
        }

        screen.handle_event(&event);
    }
}

fn get_display() -> Result<impl Display> {
    Ok(FramebufferDisplay::new()?)
}