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
    pixelcolor::Bgr888, prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle, RoundedRectangle},
    text::{Alignment, Text}
};

use crate::{
    backplate::HvacMode, drawable::{AppDrawable, AppFrameBuf},
    events::{Event, EventHandler, EventSender},
    screen_manager::Screen, theme::{IconTheme, ListTheme, ModeSelectTheme}
};

pub struct ModeScreen<S> {
    mode_icon: IconView,
    mode_list: ListView<HvacMode>,
    event_sender: S,
    highlight_row: f32,
    theme: ModeSelectTheme
}

impl<S: EventSender> ModeScreen<S> {
    pub fn new(theme: ModeSelectTheme, event_sender: S, current_mode: &HvacMode) -> Result<Self> {
        let modes = [
            HvacMode::Heat,
            HvacMode::Cool,
            HvacMode::Off
        ];

        let selected_row = modes.iter()
            .position(|m| m == current_mode)
            .unwrap_or_default();

        Ok(Self {
            mode_icon: IconView::new(theme.mode_icon.clone()),
            mode_list: ListView::new(
                theme.mode_list.clone(),
                &modes,
                selected_row
            )?,
            event_sender,
            highlight_row: selected_row as f32,
            theme
        })
    }
}

impl<S: EventSender> Screen for ModeScreen<S> { }

impl<S: EventSender> EventHandler for ModeScreen<S> {
    fn handle_event(&mut self, event: &Event) -> Result<()> {
        match event {
            Event::Dial(dir) => {
                let highlight = self.highlight_row + (*dir as f32 * 0.01);
                if self.mode_list.set_highlight_row(highlight as i32) {
                    self.highlight_row = highlight;
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

struct IconView {
    theme: IconTheme
}

impl IconView {
    fn new(theme: IconTheme) -> Self {
        Self { theme }
    }

    fn draw<D>(
        &self,
        target: &mut D,
        position: Point,
        bg_colour: Bgr888,
        fg_colour: Option<Bgr888>
    ) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let colour = fg_colour.unwrap_or(self.theme.colour);

        Text::with_alignment(
            &self.theme.icon,
            position,
            self.theme.icon_font.font_style(colour, bg_colour),
            Alignment::Center
        )
        .draw(target)?;

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
    theme: ListTheme
}

impl<T> ListView<T> {
    fn new<R>(theme: ListTheme, rows: &[R], selected_row: usize) -> Result<Self>
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
            highlight_row: selected_row
        })
    }

    fn set_highlight_row(&mut self, row: i32) -> bool {
        if row >= 0 && row < self.rows.len() as i32 {
            self.highlight_row = row as usize;
            true
        } else {
            false
        }
    }

    fn get_highlighted_value(&self) -> &T {
        let row = self.rows.get(self.highlight_row).unwrap();
        &row.value
    }

    fn get_list_size(&self) -> Size {
        Size {
            width: self.theme.row_size.width,
            height: self.rows.len() as u32 * self.theme.row_size.height
        }
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
            center.y - (self.theme.label_font.size as i32 / 2)
        );

        Text::with_alignment(
            text,
            text_pos,
            self.theme.label_font.font_style(text_color, text_bg_color),
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
        let padding = (row_rect.size.height - self.theme.icon_font.size) / 2;
        let text_pos = Point::new(
            top_left.x + padding as i32,
            top_left.y + padding as i32
        );

        Text::with_alignment(
            text,
            text_pos,
            self.theme.icon_font.font_style(text_color, text_bg_color),
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

    fn draw<D>(&self, target: &mut D, bg_colour: Bgr888) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>
    {
        let mut row_rect = Rectangle::new(Point::zero(), self.theme.row_size);
        let row_offset = Point::new(0, self.theme.row_size.height as i32);

        for (i, row) in self.rows.iter().enumerate() {
            let text_colour = if i == self.highlight_row {
                self.theme.highlight_text_colour
            } else {
                self.theme.colour
            };

            let text_bg_colour = if i == self.highlight_row {
                self.draw_highlight(target, row_rect)?;
                self.theme.highlight_rect.fill_colour
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
                    &self.theme.selected_icon
                )?;
            }

            self.draw_row_text(target, text_colour, text_bg_colour, row_rect, &row.label)?;

            row_rect = row_rect.translate(row_offset);
        }

        Ok(())
    }
}
