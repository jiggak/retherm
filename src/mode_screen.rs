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

use anyhow::{Result, anyhow};
use embedded_graphics::{
    pixelcolor::Bgr888, prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::{Alignment, Text}
};
use embedded_ttf::{FontTextStyle, FontTextStyleBuilder};
use rusttype::Font;

use crate::{
    backplate::HvacMode, drawable::{AppDrawable, AppFrameBuf},
    events::{Event, EventHandler, EventSender}, screen_manager::Screen
};

pub struct ModeScreen<S> {
    mode_list: ListView<HvacMode>,
    event_sender: S
}

impl<S: EventSender> ModeScreen<S> {
    pub fn new(event_sender: S, current_mode: &HvacMode) -> Result<Self> {
        let modes = [
            HvacMode::Heat,
            HvacMode::Cool,
            HvacMode::Off
        ];

        let selected_row = modes.iter()
            .position(|m| m == current_mode)
            .unwrap_or_default();

        Ok(Self {
            mode_list: ListView::new(&modes, selected_row)?,
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
                    let selected = self.mode_list.selected_row as i32 + inc;
                    self.mode_list.set_selected_row(selected);
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
        target.clear(Bgr888::BLACK)?;

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
    font_style: FontTextStyle<Bgr888>
}

impl<T> ListView<T> {
    fn new<R>(rows: &[R], selected_row: usize) -> Result<Self>
        where R: Clone + Into<ListItem<T>>
    {
        let font = Font::try_from_bytes(include_bytes!("../roboto/Roboto-Bold.ttf"))
            .ok_or(anyhow!("Invalid font data"))?;

        // I call clone later on font_style, is that better than re-creating
        // FontTextStyle on each render loop?
        let font_style = FontTextStyleBuilder::new(font)
            .font_size(36)
            .text_color(Bgr888::WHITE)
            .anti_aliasing_color(Bgr888::BLACK)
            .build();

        let rows = rows.iter()
            .cloned()
            .map(Into::into)
            .collect();

        Ok(Self {
            rows,
            selected_row,
            font_style
        })
    }

    fn set_selected_row(&mut self, row: i32) {
        if row >= 0 && row < self.rows.len() as i32 && row != self.selected_row as i32 {
            self.selected_row = row as usize;
        }
    }

    fn get_selected_value(&self) -> &T {
        let row = self.rows.get(self.selected_row).unwrap();
        &row.value
    }

    fn draw_row<D>(&self, target: &mut D, y_pos: usize, text: &str) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let center = target.bounding_box().center();
        let text_pos = Point::new(
            center.x,
            y_pos as i32 + 2
        );

        let text = Text::with_alignment(
            text,
            text_pos,
            self.font_style.clone(),
            Alignment::Center
        );

        text.draw(target)?;

        Ok(())
    }
}

impl<T> AppDrawable for ListView<T> {
    fn draw(&self, target: &mut AppFrameBuf) -> Result<()> {
        let (row_width, row_height): (usize, usize) = (120, 40);;
        let list_height = self.rows.len() * row_height;

        let start_x = (target.width() - row_width) / 2;
        let start_y = (target.height() - list_height) / 2;

        for (i, row) in self.rows.iter().enumerate() {
            self.draw_row(target, start_y + (i * row_height), &row.label)?;
        }

        let rect = Rectangle::new(
            Point::new(start_x as i32, (start_y + (self.selected_row * row_height)) as i32),
            Size::new(row_width as u32, row_height as u32)
        );

        let style = PrimitiveStyleBuilder::new()
            .stroke_color(Bgr888::WHITE)
            .stroke_width(4)
            .build();

        RoundedRectangle::with_equal_corners(rect, Size::new(10, 10))
            .into_styled(style)
            .draw(target)?;

        Ok(())
    }
}
