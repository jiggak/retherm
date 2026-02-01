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

use embedded_graphics::{
    pixelcolor::Bgr888,
    prelude::*,
    primitives::{Arc, Circle, PrimitiveStyle},
    text::{Alignment, Text, renderer::TextRenderer}
};

use crate::{backplate::{HvacMode, HvacState}, theme::GaugeStyle};

pub struct GaugeWidget {
    pub hvac_state: HvacState,
    style: GaugeStyle
}

impl GaugeWidget {
    pub fn new(style: GaugeStyle) -> Self {
        Self {
            hvac_state: HvacState::default(),
            style
        }
    }

    pub fn draw<D>(
        &self,
        target: &mut D,
        bg_colour: Bgr888
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let center = target.bounding_box().center();
        let target_temp_percent = get_temp_percent(self.hvac_state.target_temp);
        let current_temp_percent = get_temp_percent(self.hvac_state.current_temp);

        self.draw_temp_text(target, bg_colour, center)?;

        // gauge background
        self.draw_arc(target, 0.0, 1.0, center, self.style.arc_bg_colour)?;

        // gauge foreground
        let dot_colour = if matches!(self.hvac_state.mode, HvacMode::Heat) {
            self.draw_arc(target, 0.0, target_temp_percent, center, self.style.arc_heat_colour)?;
            self.style.arc_heat_dot_colour
        } else if matches!(self.hvac_state.mode, HvacMode::Cool) {
            self.draw_arc(target, target_temp_percent, 1.0, center, self.style.arc_cool_colour)?;
            self.style.arc_cool_dot_colour
        } else {
            self.style.arc_bg_colour
        };

        // large dot for target temp, with another dot inside
        self.draw_arc_point(target, target_temp_percent, center, self.style.arc_target_dot_dia, dot_colour)?;
        self.draw_arc_point(target, target_temp_percent, center, self.style.arc_width, self.style.fg_colour)?;

        // small dot for current temp
        self.draw_arc_point(target, current_temp_percent, center, self.style.arc_temp_dot_dia, self.style.arc_temp_dot_colour)?;

        // draw current temp label along the current temp angle
        let current_temp = format!("{:.1}", self.hvac_state.current_temp);
        let current_temp_center = self.get_arc_point(center, current_temp_percent, self.style.arc_temp_text_dia);
        self.draw_sm_text(target, bg_colour, current_temp_center, current_temp)?;

        Ok(())
    }

    fn get_arc_point(&self, center: Point, percent: f32, diameter: u32) -> Point {
        let point_angle = self.style.arc_sweed_deg * percent + self.style.arc_start_deg;
        let point_angle = Angle::from_degrees(point_angle);
        let radius = (diameter / 2) as f32;
        center + Point::new(
            (point_angle.to_radians().cos() * radius).round() as i32,
            (point_angle.to_radians().sin() * radius).round() as i32
        )
    }

    fn draw_temp_text<D>(
        &self,
        target: &mut D,
        bg_color: Bgr888,
        center: Point
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let (temp_int, temp_frac) = round_temperature(self.hvac_state.target_temp);
        let (temp_int_s, temp_frac_s) = (temp_int.to_string(), temp_frac.to_string());

        let font_style = self.style.target_font
            .font_style(self.style.fg_colour, bg_color);

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
            let font_style = self.style.target_decimal_font
                .font_style(self.style.fg_colour, bg_color);

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

    fn draw_sm_text<D>(
        &self,
        target: &mut D,
        bg_color: Bgr888,
        center: Point,
        s: String
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let font_style = self.style.current_font
            .font_style(self.style.fg_colour, bg_color);

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
        let angle_start = self.style.arc_start_deg + (self.style.arc_sweed_deg * start_percent);
        let sweep_angle = self.style.arc_sweed_deg * (end_percent - start_percent);

        let arc = Arc::with_center(
            center,
            self.style.arc_dia,
            Angle::from_degrees(angle_start),
            Angle::from_degrees(sweep_angle)
        );

        // This is most likely less efficient than arc.into_styled().draw()
        // But we get pretty rounded corners
        for p in arc.points() {
            Circle::with_center(p, self.style.arc_width)
                .into_styled(PrimitiveStyle::with_fill(colour))
                .draw(target)?;
        }

        Ok(())
    }

    fn draw_arc_point<D>(
        &self,
        target: &mut D,
        percent: f32,
        center: Point,
        dia: u32,
        colour: D::Color
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let point_center = self.get_arc_point(center, percent, self.style.arc_dia);

        Circle::with_center(point_center, dia)
            .into_styled(PrimitiveStyle::with_fill(colour))
            .draw(target)?;

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
