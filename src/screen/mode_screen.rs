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

use anyhow::Result;
use embedded_graphics::{prelude::*, primitives::Rectangle};

use crate::{
    backplate::HvacMode, drawable::{AppDrawable, AppFrameBuf},
    events::{Event, EventHandler, EventSender},
    theme::ModeSelectTheme,
    widgets::{IconWidget, ListItem, ListWidget}
};
use super::Screen;

pub struct ModeScreen<S> {
    mode_icon: IconWidget,
    mode_list: ListWidget<HvacMode>,
    event_sender: S,
    highlight_row: f32,
    theme: ModeSelectTheme
}

impl<S: EventSender> ModeScreen<S> {
    pub fn new(theme: ModeSelectTheme, event_sender: S, current_mode: &HvacMode) -> Self {
        let modes = [
            HvacMode::Heat,
            HvacMode::Cool,
            HvacMode::Off
        ];

        let selected_row = modes.iter()
            .position(|m| m == current_mode)
            .unwrap_or_default();

        Self {
            mode_icon: IconWidget::new(theme.mode_icon.clone()),
            mode_list: ListWidget::new(
                theme.mode_list.clone(),
                &modes,
                selected_row
            ),
            event_sender,
            highlight_row: selected_row as f32,
            theme
        }
    }
}

impl<S: EventSender> Screen for ModeScreen<S> { }

impl<S: EventSender> EventHandler for ModeScreen<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Dial(dir) => {
                let highlight = self.highlight_row + (*dir as f32 * 0.01);
                let last_selected = self.mode_list.get_highlight_row();

                if self.mode_list.set_highlight_row(highlight as i32) {
                    self.highlight_row = highlight;

                    if last_selected != self.mode_list.get_highlight_row() {
                        self.event_sender.send_event(Event::ClickSound)?;
                    }
                }
            }
            Event::ButtonDown => {
                let mode = self.mode_list.get_highlighted_value();
                self.event_sender.send_event(Event::SetMode(*mode))?;
                self.event_sender.send_event(Event::NavigateBack)?;
            },
            _ => { }
        }
        Ok(())
    }
}

impl<S: EventSender> AppDrawable for ModeScreen<S> {
    fn draw(&self, target: &mut AppFrameBuf) -> Result<()> {
        target.clear(self.theme.bg_colour)?;

        // draw icon view

        let icon_color = match self.mode_list.get_highlighted_value() {
            HvacMode::Heat => Some(self.theme.icon_heat_colour),
            HvacMode::Cool => Some(self.theme.icon_cool_colour),
            _ => None
        };
        self.mode_icon.draw(target, self.theme.icon_center, self.theme.bg_colour, icon_color)?;

        // draw list view

        let list_size = self.mode_list.get_list_size();
        let list_x = (target.width() as u32 - list_size.width) / 2;
        let list_y = (target.height() as u32 - list_size.height) / 2;

        let list_rect = Rectangle {
            size: list_size,
            top_left: Point {
                x: list_x as i32,
                y: list_y as i32
            }
        };

        let mut list_target = target.cropped(&list_rect);
        self.mode_list.draw(&mut list_target, self.theme.bg_colour)?;

        Ok(())
    }
}

impl From<HvacMode> for ListItem<HvacMode> {
    fn from(value: HvacMode) -> Self {
        match value {
            HvacMode::Off => ListItem {
                value: value.clone(),
                label: String::from("Off")
            },
            HvacMode::Auto => ListItem {
                value: value.clone(),
                label: String::from("Auto")
            },
            HvacMode::Heat => ListItem {
                value: value.clone(),
                label: String::from("Heat")
            },
            HvacMode::Cool => ListItem {
                value: value.clone(),
                label: String::from("Cool")
            }
        }
    }
}
