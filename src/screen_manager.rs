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

use crate::{drawable::AppDrawable, events::{Event, EventHandler}};

pub trait Screen: AppDrawable + EventHandler { }

pub struct ScreenManager {
    main_screen: Box<dyn Screen>,
    screens: Vec<Box<dyn Screen>>
}

impl ScreenManager {
    pub fn new<S: Screen + 'static>(main_screen: S) -> Self {
        Self {
            main_screen: Box::new(main_screen),
            screens: Vec::new()
        }
    }

    pub fn active_screen(&mut self) -> &mut dyn Screen {
        if let Some(screen) = self.screens.last_mut() {
            screen.as_mut()
        } else {
            self.main_screen.as_mut()
        }
    }
}

impl EventHandler for ScreenManager {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        self.active_screen().handle_event(event)?;

        // if let Event::ShowScreen(screen) = event {
        //     self.screens.push(screen);
        // }

        Ok(())
    }
}
