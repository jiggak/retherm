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

use argh::FromArgs;
use log::LevelFilter;

#[derive(FromArgs)]
/// ReTherm
pub struct Cli {
    #[argh(option)]
    /// send log level output to syslog [OFF|ERROR|WARN|INFO|DEBUG|TRACE]
    pub syslog: Option<LevelFilter>,

    #[argh(option)]
    /// path to config file
    pub config: Option<String>,

    #[argh(option)]
    /// path to theme file
    pub theme: Option<String>
}

impl Cli {
    pub fn load() -> Self {
        argh::from_env()
    }
}
