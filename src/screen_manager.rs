/*
 * Nest UI - Home Assistant native thermostat interface
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

use anyhow::Result;

use crate::{
    drawable::AppDrawable,
    events::{Event, EventHandler, EventSender},
    mode_screen::ModeScreen
};

#[derive(Debug)]
pub enum ScreenId {
    ModeSelect
}

pub trait Screen: AppDrawable + EventHandler { }

pub struct ScreenManager<S> {
    main_screen: Box<dyn Screen>,
    screens: Vec<Box<dyn Screen>>,
    event_sender: S
}

impl<S: EventSender + Clone + 'static> ScreenManager<S> {
    pub fn new<R>(main_screen: R, event_sender: S) -> Self
        where R: Screen + 'static
    {
        Self {
            main_screen: Box::new(main_screen),
            screens: Vec::new(),
            event_sender
        }
    }

    pub fn active_screen(&mut self) -> &mut dyn Screen {
        if let Some(screen) = self.screens.last_mut() {
            screen.as_mut()
        } else {
            self.main_screen.as_mut()
        }
    }

    fn show_screen(&mut self, screen: &ScreenId) -> Result<()> {
        match screen {
            ScreenId::ModeSelect => {
                let screen = ModeScreen::new(self.event_sender.clone())?;
                self.screens.push(Box::new(screen));
            }
        }

        Ok(())
    }
}

impl<S: EventSender + Clone + 'static> EventHandler for ScreenManager<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        self.active_screen().handle_event(event)?;

        match event {
            Event::NavigateTo(screen) => {
                self.show_screen(screen)?;
            }
            Event::NavigateBack => {
                self.screens.pop();
            }
            _ => { }
        }

        Ok(())
    }
}
