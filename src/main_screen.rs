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
use embedded_graphics::{
    pixelcolor::Bgr888, prelude::*, primitives::{Arc, Circle, PrimitiveStyle},
    text::{Alignment, Text, renderer::TextRenderer}
};

use crate::{
    backplate::{HvacMode, HvacState}, drawable::{AppDrawable, AppFrameBuf},
    events::{Event, EventHandler, EventSender, TrailingEventSender},
    screen_manager::{Screen, ScreenId}, theme::{FontStyle, GaugeTheme}
};

pub struct MainScreen<S> {
    gauge: ThermostatGauge,
    cmd_sender: TrailingEventSender,
    event_sender: S,
}

impl<S: EventSender> Screen for MainScreen<S> { }

impl<S: EventSender + Clone + Send + 'static> MainScreen<S> {
    pub fn new(theme: &GaugeTheme, event_sender: S) -> Result<Self> {
        let cmd_sender = TrailingEventSender::new(event_sender.clone(), 500);
        Ok(Self {
            gauge: ThermostatGauge::new(theme.clone())?,
            cmd_sender, event_sender
        })
    }
}

impl<S: EventSender> EventHandler for MainScreen<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Dial(dir) => {
                let target_temp = self.gauge.hvac_state.target_temp
                    + (*dir as f32 * 0.01);

                if self.gauge.hvac_state.set_target_temp(target_temp) {
                    self.cmd_sender.send_event(Event::SetTargetTemp(target_temp))?;
                }
            }
            Event::ButtonDown => {
                self.event_sender.send_event(Event::NavigateTo(ScreenId::ModeSelect {
                    current_mode: self.gauge.hvac_state.mode
                }))?;
            }
            Event::HvacState(state) => {
                self.gauge.hvac_state = state.clone();
            }
            _ => { }
        }

        Ok(())
    }
}

impl<S: EventSender> AppDrawable for MainScreen<S> {
    fn draw(&self, target: &mut AppFrameBuf) -> Result<()> {
        self.gauge.draw(target)?;

        Ok(())
    }
}

struct ThermostatGauge {
    hvac_state: HvacState,
    theme: GaugeTheme
}

impl ThermostatGauge {
    fn new(theme: GaugeTheme) -> Result<Self> {
        Ok(Self {
            hvac_state: HvacState::default(),
            theme
        })
    }

    fn get_arc_point(&self, center: Point, percent: f32, diameter: u32) -> Point {
        let point_angle = self.theme.arc_sweed_deg * percent + self.theme.arc_start_deg;
        let point_angle = Angle::from_degrees(point_angle);
        let radius = (diameter / 2) as f32;
        center + Point::new(
            (point_angle.to_radians().cos() * radius).round() as i32,
            (point_angle.to_radians().sin() * radius).round() as i32
        )
    }

    fn draw_temp_text<D>(&self, target: &mut D, center: Point) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let (temp_int, temp_frac) = round_temperature(self.hvac_state.target_temp);
        let (temp_int_s, temp_frac_s) = (temp_int.to_string(), temp_frac.to_string());

        let font_style = self.theme.font_style(&self.theme.target_font);

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
            let font_style = self.theme.font_style(&self.theme.target_decimal_font);

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

    fn draw_sm_text<D>(&self, target: &mut D, center: Point, s: String) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let font_style = self.theme.font_style(&self.theme.current_font);

        let text = Text::with_alignment(
            &s,
            center,
            font_style,
            Alignment::Center
        );

        text.draw(target)?;

        Ok(())
    }

    fn draw_arc<D>(
        &self,
        target: &mut D,
        start_percent: f32,
        end_percent: f32,
        center: Point,
        colour: D::Color
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let angle_start = self.theme.arc_start_deg + (self.theme.arc_sweed_deg * start_percent);
        let sweep_angle = self.theme.arc_sweed_deg * (end_percent - start_percent);

        let arc = Arc::with_center(
            center,
            self.theme.arc_dia,
            Angle::from_degrees(angle_start),
            Angle::from_degrees(sweep_angle)
        );

        // This is most likely less efficient than arc.into_styled().draw()
        // But we get pretty rounded corners
        for p in arc.points() {
            Circle::with_center(p, self.theme.arc_width)
                .into_styled(PrimitiveStyle::with_fill(colour))
                .draw(target)?;
        }

        Ok(())
    }

    fn draw_arc_point<D>(&self, target: &mut D, percent: f32, center: Point, dia: u32, colour: D::Color) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let point_center = self.get_arc_point(center, percent, self.theme.arc_dia);

        Circle::with_center(point_center, dia)
            .into_styled(PrimitiveStyle::with_fill(colour))
            .draw(target)?;

        Ok(())
    }
}

impl AppDrawable for ThermostatGauge {
    fn draw(&self, target: &mut AppFrameBuf) -> Result<()> {
        target.clear(self.theme.bg_colour)?;

        let center = target.bounding_box().center();
        let target_temp_percent = get_temp_percent(self.hvac_state.target_temp);
        let current_temp_percent = get_temp_percent(self.hvac_state.current_temp);

        self.draw_temp_text(target, center)?;

        // gauge background
        self.draw_arc(target, 0.0, 1.0, center, self.theme.arc_bg_colour)?;

        // gauge foreground
        let dot_colour = if matches!(self.hvac_state.mode, HvacMode::Heat) {
            self.draw_arc(target, 0.0, target_temp_percent, center, self.theme.arc_heat_colour)?;
            self.theme.arc_heat_dot_colour
        } else if matches!(self.hvac_state.mode, HvacMode::Cool) {
            self.draw_arc(target, target_temp_percent, 1.0, center, self.theme.arc_cool_colour)?;
            self.theme.arc_cool_dot_colour
        } else {
            self.theme.arc_bg_colour
        };

        // large dot for target temp, with another dot inside
        self.draw_arc_point(target, target_temp_percent, center, self.theme.arc_target_dot_dia, dot_colour)?;
        self.draw_arc_point(target, target_temp_percent, center, self.theme.arc_width, self.theme.fg_colour)?;

        // small dot for current temp
        self.draw_arc_point(target, current_temp_percent, center, self.theme.arc_temp_dot_dia, self.theme.arc_temp_dot_colour)?;

        // draw current temp label along the current temp angle
        let current_temp = format!("{:.1}", self.hvac_state.current_temp);
        let current_temp_center = self.get_arc_point(center, current_temp_percent, self.theme.arc_temp_text_dia);
        self.draw_sm_text(target, current_temp_center, current_temp)?;

        Ok(())
    }
}

fn round_temperature(value: f32) -> (i32, i32) {
    let scaled = (value * 2.0).round() as i32;

    let integer_part = scaled / 2;
    let fraction_part = (scaled % 2) * 5;

    (integer_part, fraction_part)
}

fn get_temp_percent(temp: f32) -> f32 {
    (temp - HvacState::MIN_TEMP) / (HvacState::MAX_TEMP - HvacState::MIN_TEMP)
}
