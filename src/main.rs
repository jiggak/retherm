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

mod backlight;
mod backplate;
mod cli;
mod config;
mod drawable;
mod events;
mod home_assistant;
mod input_events;
mod main_screen;
mod mode_screen;
mod screen_manager;
mod sound;
mod theme;
#[cfg(feature = "device")]
mod window_fb;
#[cfg(feature = "simulate")]
mod window_sdl;

use anyhow::Result;
use esphome_api::server::{EncryptedStreamProvider, PlaintextStreamProvider};
use log::debug;

use crate::backplate::{Backplate, hvac_control};
use crate::events::{Event, EventHandler, EventSender, EventSource};
use crate::home_assistant::HomeAssistant;
use crate::main_screen::MainScreen;
use crate::screen_manager::ScreenManager;

fn main() -> Result<()> {
    env_logger::init();

    let cli = cli::Cli::load();
    let config = if let Some(file_path) = cli.config {
        config::Config::load(file_path)?
    } else {
        config::Config::default()
    };

    let theme = if let Some(file_path) = cli.theme {
        theme::Theme::load(file_path)?
    } else {
        theme::Theme::default()
    };

    let mut event_source = get_event_source()?;

    let mut window = get_window(&config.backlight)?;

    let main_screen = MainScreen::new(&theme.gauge, event_source.event_sender())?;
    let mut screen_manager = ScreenManager::new(theme, main_screen, event_source.event_sender());

    start_threads(&event_source)?;

    let mut home_assistant = HomeAssistant::new();
    if let Some(key) = &config.home_assistant.encryption_key {
        let stream_factory = EncryptedStreamProvider::new(
            key,
            &config.home_assistant.node_name,
            "01:02:03:04:05:06"
        )?;

        home_assistant.start_listener(
            &config.home_assistant,
            stream_factory,
            event_source.event_sender()
        );
    } else {
        home_assistant.start_listener(
            &config.home_assistant,
            PlaintextStreamProvider::new(),
            event_source.event_sender()
        );
    }

    let hvac_control = hvac_control(event_source.event_sender())?;
    let mut backplate = Backplate::new(event_source.event_sender(), hvac_control)?;

    'running: loop {
        window.draw_screen(screen_manager.active_screen())?;

        let event = event_source.wait_event()?;
        if matches!(event, Event::Quit) {
            break 'running;
        }

        debug!("{:?}", event);

        let handlers: [&mut dyn EventHandler; _] = [
            &mut window, &mut screen_manager, &mut home_assistant, &mut backplate
        ];

        for handler in handlers {
            handler.handle_event(&event)?;
        }
    }

    Ok(())
}

#[cfg(feature = "device")]
fn get_window(config: &config::BacklightConfig) -> Result<crate::window_fb::FramebufferWindow> {
    crate::window_fb::FramebufferWindow::new(config)
}

#[cfg(feature = "device")]
fn get_event_source() -> Result<crate::events::DefaultEventSource> {
    Ok(crate::events::DefaultEventSource::new())
}

#[cfg(feature = "device")]
fn start_threads<E, S>(events: &E) -> Result<()>
    where E: EventSource<S>, S: EventSender + Send + 'static
{
    crate::input_events::start_button_events(events.event_sender())?;
    // Slow down events from dial to make it feel less twitchy
    // And spam the event loop less
    let dial_event_sender = crate::events::ThrottledEventSender::new(events.event_sender(), 40, 1);
    crate::input_events::start_dial_events(dial_event_sender)?;
    Ok(())
}

#[cfg(feature = "simulate")]
fn get_window(_config: &config::BacklightConfig) -> Result<crate::window_sdl::SdlWindow> {
    crate::window_sdl::SdlWindow::new()
}

#[cfg(feature = "simulate")]
fn get_event_source() -> Result<crate::window_sdl::SdlEventSource> {
    crate::window_sdl::SdlEventSource::new()
}

#[cfg(feature = "simulate")]
fn start_threads<E: EventSource<S>, S: EventSender>(_events: &E) -> Result<()> {
    Ok(())
}
