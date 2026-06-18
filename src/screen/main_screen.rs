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

use std::time::Duration;

use anyhow::Result;
use embedded_graphics::{
    pixelcolor::Bgr888,
    prelude::*,
    text::{Alignment, Text, renderer::TextRenderer}
};

use crate::{
    drawable::{AppDrawable, AppFrameBuf},
    events::{Event, EventHandler, EventSender, TrailingEventSender},
    state::{HvacAction, HvacMode, ThermostatState},
    theme::MainScreenTheme,
    timer::TimerId,
    widgets::{GaugeWidget, IconWidget}
};
use super::{Screen, ScreenId};

pub struct MainScreen<S> {
    gauge: GaugeWidget,
    away_icon: IconWidget,
    lockout_icon: IconWidget,
    cmd_sender: TrailingEventSender,
    event_sender: S,
    theme: MainScreenTheme,
    state: ThermostatState,
    last_click_temp: f32,
    scrolling: bool,
}

impl<S: EventSender> Screen for MainScreen<S> { }

impl<S: EventSender + Clone + Send + 'static> MainScreen<S> {
    pub fn new(theme: MainScreenTheme, state: ThermostatState, event_sender: S) -> Self {
        let cmd_sender = TrailingEventSender::new(event_sender.clone(), 250, Event::DialCommit);
        Self {
            gauge: GaugeWidget::new(theme.gauge.clone()),
            away_icon: IconWidget::new(theme.away_icon.clone()),
            lockout_icon: IconWidget::new(theme.lockout_icon.clone()),
            cmd_sender,
            event_sender,
            theme,
            state,
            last_click_temp: 0.0,
            scrolling: false,
        }
    }
}

impl<S: EventSender> EventHandler for MainScreen<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        // Ignore button and dial events while in away mode
        // Let state manager exit away mode before

        match event {
            Event::Dial(dir) if !self.state.away => {
                self.scrolling = true;

                let temp_inc = *dir as f32 * 0.01;
                let target_temp = self.state.target_temp + temp_inc;

                if (self.last_click_temp - target_temp).abs() >= 0.5 {
                    self.last_click_temp = target_temp;
                    self.event_sender.send_event(Event::ClickSound)?;
                }

                if self.state.set_target_temp(target_temp) {
                    self.cmd_sender.send_event(Event::SetTargetTemp(target_temp))?;
                }
            }
            Event::ButtonDown if !self.state.away => {
                self.event_sender.send_event(Event::NavigateTo(ScreenId::ModeSelect {
                    current_mode: self.state.mode
                }))?;
            }
            Event::DialCommit => {
                self.scrolling = false;
            }
            // By handling lockout timer ticks here, instead of state manager
            // handling and sending `State` events, we avoid the `State` events
            // interfering with dial events.
            Event::TimerTick(TimerId::HvacLockout, remaining) => {
                if self.state.lockout.is_some() {
                    self.state.lockout = Some(*remaining);
                }
            }
            // Ignore state changes while dial scrolling to avoid contention with
            // delayed dial commit (event sent after delay of dial inactivity)
            Event::State(state) if !self.scrolling => {
                self.state = state.clone();
            }
            _ => { }
        }

        Ok(())
    }
}

impl<S: EventSender> AppDrawable for MainScreen<S> {
    fn draw(&self, target: &mut AppFrameBuf) -> Result<()> {
        let bg_colour = match self.state.action {
            HvacAction::Cooling => self.theme.bg_cool_colour,
            HvacAction::Heating => self.theme.bg_heat_colour,
            HvacAction::Idle => self.theme.bg_colour
        };

        target.clear(bg_colour)?;

        let center = target.bounding_box().center();
        self.draw_temp_text(target, bg_colour, center, self.state.target_temp)?;

        let gauge_accent = match self.state.mode {
            HvacMode::Cool => Some(&self.theme.cool_gauge),
            HvacMode::Heat => Some(&self.theme.heat_gauge),
            _ => None
        };

        let target_temp_percent = ThermostatState::temp_percent(self.state.target_temp);
        let current_temp_percent = ThermostatState::temp_percent(self.state.current_temp);
        let current_temp_label = format!("{:.1}", self.state.current_temp);

        self.gauge.draw(
            target,
            bg_colour,
            gauge_accent,
            target_temp_percent,
            current_temp_percent,
            current_temp_label
        )?;

        if self.state.away {
            self.away_icon.draw(
                target,
                self.theme.status_icon_center,
                bg_colour,
                Some(self.theme.away_icon.colour)
            )?;
        } else if let Some(lockout_duration) = self.state.lockout {
            self.lockout_icon.draw(
                target,
                self.theme.status_icon_center,
                bg_colour,
                Some(self.theme.lockout_icon.colour)
            )?;

            let dur_text = format_duration(lockout_duration);
            self.draw_status_text(target, bg_colour, dur_text)?;
        }

        Ok(())
    }
}

impl<S> MainScreen<S> {
    fn draw_status_text<D>(
        &self,
        target: &mut D,
        bg_colour: Bgr888,
        s: String
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let font_style = self.theme.status_msg_font
            .font_style(self.theme.fg_colour, bg_colour);

        let text = Text::with_alignment(
            &s,
            self.theme.status_msg_center,
            font_style,
            Alignment::Center
        );

        text.draw(target)?;

        Ok(())
    }

    fn draw_temp_text<D>(
        &self,
        target: &mut D,
        bg_color: Bgr888,
        center: Point,
        target_temp: f32
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let (temp_int, temp_frac) = round_temperature(target_temp);
        let (temp_int_s, temp_frac_s) = (temp_int.to_string(), temp_frac.to_string());

        let font_style = self.theme.target_font
            .font_style(self.theme.fg_colour, bg_color);

        let text_pos = Point::new(
            center.x,
            center.y - font_style.line_height() as i32 / 2
        );

        let text = Text::with_alignment(
            &temp_int_s,
            text_pos,
            font_style,
            Alignment::Center
        );

        text.draw(target)?;

        if temp_frac > 0 {
            let font_style = self.theme.target_decimal_font
                .font_style(self.theme.fg_colour, bg_color);

            let text_pos = Point::new(
                center.x + (text.bounding_box().size.width / 2) as i32,
                text_pos.y + font_style.line_height() as i32 / 2
            );

            let text = Text::with_alignment(
                &temp_frac_s,
                text_pos,
                font_style,
                Alignment::Left
            );

            text.draw(target)?;
        }

        Ok(())
    }
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;

    format!("{:02}:{:02}", minutes, seconds)
}

fn round_temperature(value: f32) -> (i32, i32) {
    let scaled = (value * 2.0).round() as i32;

    let integer_part = scaled / 2;
    let fraction_part = (scaled % 2) * 5;

    (integer_part, fraction_part)
}
