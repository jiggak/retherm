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

use embedded_graphics::{pixelcolor::Bgr888, prelude::DrawTarget};

/*
Do I really gain anything with this trait vs. using embedded_graphics::Drawable?
One advantage is implementing the AppDrawable trait is tiny bit cleaner:

```
impl AppDrawable for ThermostatGauge {
    fn draw<D>(&self, target: &mut D) -> std::result::Result<(), D::Error>
            where D: DrawTarget<Color = Bgr888> {
        Ok(())
    }
}
```

vs.

```
impl Drawable for ThermostatGauge {
    type Color = Bgr888;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> std::result::Result<Self::Output, D::Error>
        where
            D: DrawTarget<Color = Self::Color> {
        Ok(())
    }
}
```

The hope was by using this trait I might somehow make it easier on myself if
I ever decide to replace embedded-graphics crate. Or maybe I might add other
parameters to the draw function.
*/

pub trait AppDrawable {
    fn draw<D>(&self, target: &mut D) -> Result<(), D::Error>
        where D: DrawTarget<Color = Bgr888>;
}
