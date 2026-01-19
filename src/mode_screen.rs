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
use embedded_graphics::{
    pixelcolor::Bgr888, prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::{Alignment, Text}
};

use crate::{
    backplate::HvacMode, drawable::{AppDrawable, AppFrameBuf},
    events::{Event, EventHandler, EventSender},
    screen_manager::Screen, theme::ModeSelectTheme
};

pub struct ModeScreen<S> {
    mode_list: ListView<HvacMode>,
    event_sender: S
}

impl<S: EventSender> ModeScreen<S> {
    pub fn new(theme: &ModeSelectTheme, event_sender: S, current_mode: &HvacMode) -> Result<Self> {
        let modes = [
            HvacMode::Heat,
            HvacMode::Cool,
            HvacMode::Off
        ];

        let selected_row = modes.iter()
            .position(|m| m == current_mode)
            .unwrap_or_default();

        Ok(Self {
            mode_list: ListView::new(theme.clone(), &modes, selected_row)?,
            event_sender
        })
    }
}

impl<S: EventSender> Screen for ModeScreen<S> { }

impl<S: EventSender> EventHandler for ModeScreen<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Dial(dir) => {
                if *dir != 0 {
                    let inc = dir / dir.abs();
                    let highlight = self.mode_list.highlight_row as i32 + inc;
                    self.mode_list.set_highlight_row(highlight);
                }
            }
            Event::ButtonDown => {
                let mode = self.mode_list.get_selected_value();
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
        self.mode_list.draw(target)?;

        Ok(())
    }
}

struct ListItem<T> {
    value: T,
    label: String
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

struct ListView<T> {
    rows: Vec<ListItem<T>>,
    selected_row: usize,
    highlight_row: usize,
    theme: ModeSelectTheme
}

impl<T> ListView<T> {
    fn new<R>(theme: ModeSelectTheme, rows: &[R], selected_row: usize) -> Result<Self>
        where R: Clone + Into<ListItem<T>>
    {
        let rows = rows.iter()
            .cloned()
            .map(Into::into)
            .collect();

        Ok(Self {
            theme,
            rows,
            selected_row,
            highlight_row: 0
        })
    }

    fn set_highlight_row(&mut self, row: i32) {
        if row >= 0 && row < self.rows.len() as i32 && row != self.highlight_row as i32 {
            self.highlight_row = row as usize;
        }
    }

    fn get_selected_value(&self) -> &T {
        let row = self.rows.get(self.highlight_row).unwrap();
        &row.value
    }

    fn draw_row_text<D>(&self, target: &mut D, text_color: Bgr888, row_rect: Rectangle, text: &str) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let center = row_rect.center();
        let text_pos = Point::new(
            center.x,
            center.y - (self.theme.label_font.size as i32 / 2)
        );

        Text::with_alignment(
            text,
            text_pos,
            self.theme.label_font.font_style(text_color, self.theme.bg_colour),
            Alignment::Center
        )
        .draw(target)?;

        Ok(())
    }

    fn draw_checkmark<D>(&self, target: &mut D, text_color: Bgr888, row_rect: Rectangle, text: &str) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let top_left = row_rect.top_left;
        let padding = (row_rect.size.height - self.theme.icon_font.size) / 2;
        let text_pos = Point::new(
            top_left.x + padding as i32,
            top_left.y + padding as i32
        );

        Text::with_alignment(
            text,
            text_pos,
            self.theme.icon_font.font_style(text_color, self.theme.bg_colour),
            Alignment::Left
        )
        .draw(target)?;

        Ok(())
    }

    fn draw_highlight<D>(&self, target: &mut D, row_rect: Rectangle) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let mut style_builder = PrimitiveStyleBuilder::new();

        if let Some(color) = self.theme.highlight_rect.stroke_colour {
            style_builder = style_builder.stroke_color(color);
        }

        if let Some(width) = self.theme.highlight_rect.stroke_width {
            style_builder = style_builder.stroke_width(width);
        }

        if let Some(fill) = self.theme.highlight_rect.fill_colour {
            style_builder = style_builder.fill_color(fill);
        }

        let rect_style = style_builder.build();

        if let Some(radius) = self.theme.highlight_rect.corner_radius {
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

impl<T> AppDrawable for ListView<T> {
    fn draw(&self, target: &mut AppFrameBuf) -> Result<()> {
        target.clear(self.theme.bg_colour)?;

        let (row_width, row_height) = (
            self.theme.row_size.width as usize, self.theme.row_size.height as usize
        );
        let list_height = self.rows.len() * row_height;

        let start_x = (target.width() - row_width) / 2;
        let start_y = (target.height() - list_height) / 2;

        let mut row_rect = Rectangle::new(
            Point::new(start_x as i32, start_y as i32),
            self.theme.row_size
        );

        for (i, row) in self.rows.iter().enumerate() {
            let text_colour = if i == self.highlight_row {
                self.theme.highlight_text_colour
            } else {
                self.theme.fg_colour
            };

            self.draw_row_text(target, text_colour, row_rect, &row.label)?;

            if i == self.selected_row {
                self.draw_checkmark(target, text_colour, row_rect, &self.theme.checkmark)?;
            }

            if i == self.highlight_row {
                self.draw_highlight(target, row_rect)?;
            }

            row_rect = row_rect.translate(Point::new(0, row_height as i32));
        }

        Ok(())
    }
}
