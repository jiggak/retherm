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

mod backplate;
mod cli;
mod config;
mod drawable;
mod events;
mod home_assistant;
mod input_events;
mod screen;
mod sound;
mod state;
mod theme;
mod timer;
mod widgets;
mod window;

use anyhow::Result;
use esphome_api::server::{EncryptedStreamProvider, PlaintextStreamProvider};
use log::debug;

use crate::events::{Event, EventHandler, EventSource};
use crate::home_assistant::HomeAssistant;
use crate::screen::{MainScreen, ScreenManager};

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

    let mut event_source = window::new_event_source()?;

    let mut state_manager = state::StateManager::new(&config, event_source.event_sender())?;
    let mut backplate = backplate::Backplate::new(&config, event_source.event_sender())?;
    let mut timers = timer::Timers::new(event_source.event_sender());
    let mut sound = sound::Sound::new()?;

    let mut window = window::new_window(&config.backlight)?;

    let main_screen = MainScreen::new(theme.thermostat.clone(), event_source.event_sender());
    let mut screen_manager = ScreenManager::new(theme, main_screen, event_source.event_sender());

    input_events::start_threads(&event_source)?;

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

    'running: loop {
        window.draw_screen(screen_manager.active_screen())?;

        let event = event_source.wait_event()?;
        if matches!(event, Event::Quit) {
            break 'running;
        }

        debug!("{:?}", event);

        let handlers: [&mut dyn EventHandler; _] = [
            &mut state_manager,
            &mut backplate,
            &mut timers,
            &mut sound,
            &mut window,
            &mut screen_manager,
            &mut home_assistant
        ];

        for handler in handlers {
            handler.handle_event(&event)?;
        }
    }

    Ok(())
}
