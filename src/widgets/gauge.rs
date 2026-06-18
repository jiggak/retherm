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
    text::{Alignment, Text}
};

use crate::theme::{ArcFill, GaugeAccentStyle, GaugeStyle};

pub struct GaugeWidget {
    style: GaugeStyle
}

impl GaugeWidget {
    pub fn new(style: GaugeStyle) -> Self {
        Self { style }
    }

    pub fn draw<D>(
        &self,
        target: &mut D,
        bg_colour: Bgr888,
        accent: Option<&GaugeAccentStyle>,
        target_percent: f32,
        current_percent: f32,
        current_label: String
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let center = target.bounding_box().center();

        // gauge background arc
        self.draw_arc(target, 0.0, 1.0, center, self.style.arc_bg_colour)?;

        // gauge accent arc
        let dot_colour = if let Some(accent) = accent {
            let (arc_start, arc_end) = match accent.arc_fill {
                ArcFill::Below => (0.0, target_percent),
                ArcFill::Above => (target_percent, 1.0),
            };

            self.draw_arc(target, arc_start, arc_end, center, accent.arc_colour)?;
            accent.arc_dot_colour
        } else {
            self.style.arc_bg_colour
        };

        // large dot for target value, with another dot inside
        self.draw_arc_point(target, target_percent, center, self.style.arc_target_dot_dia, dot_colour)?;
        self.draw_arc_point(target, target_percent, center, self.style.arc_width, self.style.fg_colour)?;

        // small dot for current value
        self.draw_arc_point(target, current_percent, center, self.style.arc_dot_dia, self.style.arc_dot_colour)?;

        // draw label near current value dot
        let current_value_center = self.get_arc_point(center, current_percent, self.style.arc_text_dia);
        self.draw_text(target, bg_colour, current_value_center, current_label)?;

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

    fn draw_text<D>(
        &self,
        target: &mut D,
        bg_color: Bgr888,
        center: Point,
        s: String
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let font_style = self.style.font
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
