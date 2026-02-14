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
use embedded_graphics::{prelude::*};

use crate::{
    drawable::{AppDrawable, AppFrameBuf},
    events::{Event, EventHandler, EventSender, TrailingEventSender},
    state::HvacAction, theme::MainScreenTheme, widgets::GaugeWidget
};
use super::{Screen, ScreenId};

pub struct MainScreen<S> {
    gauge: GaugeWidget,
    cmd_sender: TrailingEventSender,
    event_sender: S,
    theme: MainScreenTheme,
    last_click_temp: f32
}

impl<S: EventSender> Screen for MainScreen<S> { }

impl<S: EventSender + Clone + Send + 'static> MainScreen<S> {
    pub fn new(theme: MainScreenTheme, event_sender: S) -> Self {
        let cmd_sender = TrailingEventSender::new(event_sender.clone(), 500);
        Self {
            gauge: GaugeWidget::new(theme.gauge.clone()),
            cmd_sender, event_sender, theme, last_click_temp: 0.0
        }
    }
}

impl<S: EventSender> EventHandler for MainScreen<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Dial(dir) => {
                let temp_inc = *dir as f32 * 0.01;
                let target_temp = self.gauge.state.target_temp + temp_inc;

                if (self.last_click_temp - target_temp).abs() >= 0.5 {
                    self.last_click_temp = target_temp;
                    self.event_sender.send_event(Event::ClickSound)?;
                }

                if self.gauge.state.set_target_temp(target_temp) {
                    self.cmd_sender.send_event(Event::SetTargetTemp(target_temp))?;
                }
            }
            Event::ButtonDown => {
                self.event_sender.send_event(Event::NavigateTo(ScreenId::ModeSelect {
                    current_mode: self.gauge.state.mode
                }))?;
            }
            Event::State(state) => {
                self.gauge.state = state.clone();
            }
            _ => { }
        }

        Ok(())
    }
}

impl<S: EventSender> AppDrawable for MainScreen<S> {
    fn draw(&self, target: &mut AppFrameBuf) -> Result<()> {
        let bg_colour = match self.gauge.state.action {
            HvacAction::Cooling => self.theme.bg_cool_colour,
            HvacAction::Heating => self.theme.bg_heat_colour,
            HvacAction::Idle => self.theme.bg_colour
        };

        target.clear(bg_colour)?;

        self.gauge.draw(target, bg_colour)?;

        Ok(())
    }
}
