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

use anyhow::{Result, anyhow};
use embedded_graphics::{
    pixelcolor::Bgr888, prelude::*, primitives::{Arc, Circle, PrimitiveStyle},
    text::{Alignment, Text}
};
use embedded_ttf::FontTextStyleBuilder;
use rusttype::Font;

use crate::{
    backplate::HvacState, drawable::{AppDrawable, AppFrameBuf},
    events::{Event, EventHandler, EventSender}
};

pub struct MainScreen<S> {
    gauge: ThermostatGauge,
    event_sender: S
}

impl<S: EventSender> MainScreen<S> {
    pub fn new(event_sender: S) -> Result<Self> {
        Ok(Self {
            gauge: ThermostatGauge::new()?,
            event_sender
        })
    }
}

impl<S: EventSender> EventHandler for MainScreen<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Dial(dir) => {
                let mut target_temp = self.gauge.hvac_state.target_temp;
                if *dir > 0 {
                    target_temp = target_temp + 0.1;
                } else if *dir < 0 {
                    target_temp = target_temp - 0.1;
                }

                if self.gauge.hvac_state.set_target_temp(target_temp) {
                    self.event_sender.send_event(Event::SetTargetTemp(target_temp))?;
                }
            },
            Event::HvacState(state) => {
                self.gauge.hvac_state = state.clone();
            },
            _ => { }
        }

        Ok(())
    }
}

impl<S: EventSender> AppDrawable for MainScreen<S> {
    fn draw(&self, target: &mut AppFrameBuf) -> Result<()> {
        target.clear(Bgr888::BLACK)?;

        self.gauge.draw(target)?;

        Ok(())
    }
}

struct ThermostatGauge {
    hvac_state: HvacState,
    font_reg: Font<'static>,
    font_bold: Font<'static>
}

impl ThermostatGauge {
    const FONT_SIZE_LG: u32 = 100;
    const FONT_SIZE_MD: u32 = 40;
    const FONT_SIZE_SM: u32 = 20;
    const FONT_FG_COLOUR: Bgr888 = Bgr888::WHITE;
    const FONT_BG_COLOUR: Bgr888 = Bgr888::BLACK;

    const ARC_DIA: u32 = 280;
    const ARC_WIDTH: u32 = 12;
    // Arc start angle starts at 3'oclock
    const ARC_START_DEG: f32 = 120.0;
    const ARC_SWEEP_DEG: f32 = 300.0;

    fn new() -> Result<Self> {
        // I have no idea if it makes sense to keep this as a struct variable.
        // It feels like a bad idea to be re-loading fonts each time a draw is
        // required. At some point the fonts will be loaded from files specified
        // in configuration files, so some sort of resource manager might be
        // required.
        let font_reg = Font::try_from_bytes(include_bytes!("../roboto/Roboto-Regular.ttf"))
            .ok_or(anyhow!("Invalid font data"))?;

        let font_bold = Font::try_from_bytes(include_bytes!("../roboto/Roboto-Bold.ttf"))
            .ok_or(anyhow!("Invalid font data"))?;

        Ok(Self {
            hvac_state: HvacState::default(),
            font_reg,
            font_bold
        })
    }

    fn get_arc_point(center: Point, percent: f32, radius: f32) -> Point {
        let point_angle = Self::ARC_SWEEP_DEG * percent + Self::ARC_START_DEG;
        let point_angle = Angle::from_degrees(point_angle);
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

        // is clone() better than re-loading `Font` instance?
        let font_style = FontTextStyleBuilder::new(self.font_bold.clone())
            .font_size(Self::FONT_SIZE_LG)
            .text_color(Self::FONT_FG_COLOUR)
            .anti_aliasing_color(Self::FONT_BG_COLOUR)
            .build();

        let text_pos = Point::new(
            center.x,
            center.y - Self::FONT_SIZE_LG as i32 / 2
        );

        let text = Text::with_alignment(
            &temp_int_s,
            text_pos,
            font_style,
            Alignment::Center
        );

        text.draw(target)?;

        if temp_frac > 0 {
            let font_style = FontTextStyleBuilder::new(self.font_bold.clone())
                .font_size(Self::FONT_SIZE_MD)
                .text_color(Self::FONT_FG_COLOUR)
                .anti_aliasing_color(Self::FONT_BG_COLOUR)
                .build();

            let text_pos = Point::new(
                center.x + (text.bounding_box().size.width / 2) as i32,
                text_pos.y + Self::FONT_SIZE_MD as i32 / 2
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
        let font_style = FontTextStyleBuilder::new(self.font_reg.clone())
            .font_size(Self::FONT_SIZE_SM)
            .text_color(Self::FONT_FG_COLOUR)
            .anti_aliasing_color(Self::FONT_BG_COLOUR)
            .build();

        let text = Text::with_alignment(
            &s,
            center,
            font_style,
            Alignment::Center
        );

        text.draw(target)?;

        Ok(())
    }

    fn draw_arc<D>(&self, target: &mut D, percent: f32, center: Point, colour: D::Color) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let arc = Arc::with_center(
            center,
            Self::ARC_DIA,
            Angle::from_degrees(Self::ARC_START_DEG),
            Angle::from_degrees(Self::ARC_SWEEP_DEG * percent)
        );

        // This is most likely less efficient than arc.into_styled().draw()
        // But we get pretty rounded corners
        for p in arc.points() {
            Circle::with_center(p, Self::ARC_WIDTH)
                .into_styled(PrimitiveStyle::with_fill(colour))
                .draw(target)?;
        }

        Ok(())
    }

    fn draw_arc_point<D>(&self, target: &mut D, percent: f32, center: Point, dia: u32, colour: D::Color) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let point_center = Self::get_arc_point(center, percent, (Self::ARC_DIA/2) as f32);

        Circle::with_center(point_center, dia)
            .into_styled(PrimitiveStyle::with_fill(colour))
            .draw(target)?;

        Ok(())
    }
}

impl AppDrawable for ThermostatGauge {
    fn draw(&self, target: &mut AppFrameBuf) -> Result<()> {
        let center = target.bounding_box().center();
        let target_temp_percent = get_temp_percent(self.hvac_state.target_temp);
        let current_temp_percent = get_temp_percent(self.hvac_state.current_temp);

        self.draw_temp_text(target, center)?;

        // gauge background
        self.draw_arc(target, 1.0, center, Bgr888::CSS_DIM_GRAY)?;
        // gauge foreground
        self.draw_arc(target, target_temp_percent, center, Bgr888::CSS_PERU)?;

        self.draw_arc_point(target, target_temp_percent, center, 20, Bgr888::CSS_DARK_ORANGE)?;
        self.draw_arc_point(target, target_temp_percent, center, Self::ARC_WIDTH, Bgr888::WHITE)?;

        self.draw_arc_point(target, current_temp_percent, center, 10, Bgr888::CSS_SILVER)?;

        let current_temp = format!("{:.1}", self.hvac_state.current_temp);
        let current_temp_center = Self::get_arc_point(center, current_temp_percent, 124.0);
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
