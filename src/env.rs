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

pub fn get_mac_addr() -> Result<Option<String>> {
    use nix::ifaddrs::*;

    for if_addr in getifaddrs()? {
        if let Some(addr) = if_addr.address {
            if let Some(link) = addr.as_link_addr() {
                if let Some(bytes) = link.addr() {
                    if bytes.iter().any(|b| *b != 0) {
                        let mac = bytes.iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<Vec<_>>()
                            .join(":");
                        return Ok(Some(mac))
                    }
                }
            }
        }
    }

    Ok(None)
}

pub fn get_hostname() -> Result<String> {
    use nix::unistd::gethostname;

    let hostname = gethostname()?;
    let hostname = hostname.into_string()
        .expect("hostname should be valid utf8");

    Ok(hostname)
}

pub fn get_pkg_ver() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn get_pkg_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}
