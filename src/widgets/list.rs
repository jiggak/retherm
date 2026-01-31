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
    primitives::{Rectangle, RoundedRectangle},
    text::{Alignment, Text}
};

use crate::theme::ListStyle;

pub struct ListItem<T> {
    pub value: T,
    pub label: String
}

pub struct ListWidget<T> {
    rows: Vec<ListItem<T>>,
    selected_row: usize,
    highlight_row: usize,
    style: ListStyle
}

impl<T> ListWidget<T> {
    pub fn new<R>(style: ListStyle, rows: &[R], selected_row: usize) -> Self
        where R: Clone + Into<ListItem<T>>
    {
        let rows = rows.iter()
            .cloned()
            .map(Into::into)
            .collect();

        Self {
            style,
            rows,
            selected_row,
            highlight_row: selected_row
        }
    }

    pub fn set_highlight_row(&mut self, row: i32) -> bool {
        if row >= 0 && row < self.rows.len() as i32 {
            self.highlight_row = row as usize;
            true
        } else {
            false
        }
    }

    pub fn get_highlighted_value(&self) -> &T {
        let row = self.rows.get(self.highlight_row).unwrap();
        &row.value
    }

    pub fn get_list_size(&self) -> Size {
        Size {
            width: self.style.row_size.width,
            height: self.rows.len() as u32 * self.style.row_size.height
        }
    }

    pub fn draw<D>(&self, target: &mut D, bg_colour: Bgr888) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let mut row_rect = Rectangle::new(Point::zero(), self.style.row_size);
        let row_offset = Point::new(0, self.style.row_size.height as i32);

        for (i, row) in self.rows.iter().enumerate() {
            let text_colour = if i == self.highlight_row {
                self.style.highlight_text_colour
            } else {
                self.style.colour
            };

            let text_bg_colour = if i == self.highlight_row {
                self.draw_highlight(target, row_rect)?;
                self.style.highlight_rect.fill_colour
                    .unwrap_or(bg_colour)
            } else {
                bg_colour
            };

            if i == self.selected_row {
                self.draw_selected_icon(
                    target,
                    text_colour,
                    text_bg_colour,
                    row_rect,
                    &self.style.selected_icon
                )?;
            }

            self.draw_row_text(target, text_colour, text_bg_colour, row_rect, &row.label)?;

            row_rect = row_rect.translate(row_offset);
        }

        Ok(())
    }

    fn draw_row_text<D>(
        &self,
        target: &mut D,
        text_color: Bgr888,
        text_bg_color: Bgr888,
        row_rect: Rectangle,
        text: &str
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let center = row_rect.center();
        let text_pos = Point::new(
            center.x,
            center.y - (self.style.label_font.size as i32 / 2)
        );

        Text::with_alignment(
            text,
            text_pos,
            self.style.label_font.font_style(text_color, text_bg_color),
            Alignment::Center
        )
        .draw(target)?;

        Ok(())
    }

    fn draw_selected_icon<D>(
        &self,
        target: &mut D,
        text_color: Bgr888,
        text_bg_color: Bgr888,
        row_rect: Rectangle,
        text: &str
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let top_left = row_rect.top_left;
        let padding = (row_rect.size.height - self.style.icon_font.size) / 2;
        let text_pos = Point::new(
            top_left.x + padding as i32,
            top_left.y + padding as i32
        );

        Text::with_alignment(
            text,
            text_pos,
            self.style.icon_font.font_style(text_color, text_bg_color),
            Alignment::Left
        )
        .draw(target)?;

        Ok(())
    }

    fn draw_highlight<D>(&self, target: &mut D, row_rect: Rectangle) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let rect_style = self.style.highlight_rect.rect_style();

        if self.style.highlight_rect.corner_radius > 0 {
            let radius = self.style.highlight_rect.corner_radius;
            RoundedRectangle::with_equal_corners(row_rect, Size::new_equal(radius))
                .into_styled(rect_style)
                .draw(target)?;
        } else {
            row_rect.into_styled(rect_style)
                .draw(target)?;
        }

        Ok(())
    }
}
