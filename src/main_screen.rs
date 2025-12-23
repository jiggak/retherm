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

use embedded_graphics::prelude::*;
use embedded_graphics::{pixelcolor::Bgr888, primitives::{Circle, PrimitiveStyle}};

pub struct MainScreen { }

impl Drawable for MainScreen {
    // FIXME having `Bgr888` here might be a problem for running simulator
    // I should probably make this a type alias since I will have many screens
    type Color = Bgr888;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
        where D: DrawTarget<Color = Self::Color>
    {
        let circle = Circle::new(Point::new(100, 100), 100)
            .into_styled(PrimitiveStyle::with_stroke(Self::Color::BLUE, 5));
        circle.draw(target)?;

        Ok(())
    }
}
