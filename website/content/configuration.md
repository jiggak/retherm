+++
title = "Configuration"
template = "docgen.html"

[extra]
toc = true

# DO NOT EDIT
# Use `make_docs.sh` to generate content
+++
# Config file

Launch retherm with the path to your custom configuration.

```bash
retherm --config ./your_config.toml
```

All config options have a default; you only need to include options
you would like to override in your configuration file.

## temp_differential

Set the amount past the target temperature before switching the hvac
wires. This is intended to reduce the frequency of switching the hvac
system on and off, to reduce wear and conserve energy.

For example with a target heat temp. of 20, and a temp. diff set to 0.2,
the hvac system will turn heat off when temp. reaches 20.2, and turns heat
on when temp. drops to 19.8.

Defaults to 0.2

# Away Mode

```toml
[away_mode]
temp_heat = 16.0
temp_cool = 20.0
timeout = "0s"
```

## temp_heat

Away temp for heating mode, default 16.0

## temp_cool

Away temp for cooling mode, default 22.0

## timeout

Duration of no proximity movement before going into away mode,
or set to zero to disable away mode. Default "30m".

# Backplate

```toml
[backplate]
near_pir_threshold = 15
serial_port = "/dev/ttyO2"
wiring = { heat_wire: "W1", cool_wire: "Y1" }
```

## near_pir_threshold

Minimum near proximity value to be considered as movement, default 15

## serial_port

Path to backplate serial device file, default "/dev/ttyO2"

## wiring

HVAC wiring configuration, default `{ heat_wire: "W1", cool_wire: "Y1" }`.
Valid wire names: W1, Y1, G, OB, W2, Y2, Star.

# Home Assistant

```toml
[home_assistant]
friendly_name = "Hallway"
encryption_key = "..."
```

## object_id

Object ID used internall by home assistant.
Defaults to "climage.{node_name}".

## listen_addr

Listen address for ESP Home API server, default "0.0.0.0:6053"

## encryption_key

Encryption key as 32 byte base64 string. When not provided, the
connection uses plaintext messages.
See [ESP Home Native API](https://esphome.io/components/api/)
for a tool that generates a random key.

## server_info

Server info (not typically displayed in Home Assistant).
Defaults to "ReTherm {version}".

## node_name

Node name, defaults to the system hostname

## friendly_name

Friendly name displayed in as label for thermostat control

## manufacturer

Manufactuer name, defaults to "Nest"

## model

Model name, defaults to "Gen2 Thermostat"

## mac_address

Mac address, defaults to address of system interface address

# Backlight

```toml
[backlight]
brightness = 108
timeout = "15s"
```

## brightness

Screen brightness, defaults to 108 (max 120)

## timeout

Timeout before screen turns off, defaults to "15s"

# Schedule

```toml
[[schedule_heat]]
days_of_week = "EveryDay"
set_points = [
   { time = "08:00", temp = 20.0 },
   { time = "22:00", temp = 16.0 },
]
```

* Heating schedule `[[schedule_heat]]`
* Cooling schedule `[[schedule_cool]]`

You can define more than one schedule entry, and it will overlap the
previous. In the example below, the temperature will be set to 20.0
at 8am everyday, and set down to 16.0 at 9am Monday and Wednsday.

```toml
[[schedule_heat]]
days_of_week = "EveryDay"
set_points = [
   { time = "08:00", temp = 20.0 }
]

[[schedule_heat]]
days_of_week = ["Monday", "Wednsday"]
set_points = [
   { time = "09:00", temp = 16.0 }
]
```

## days_of_week

Days of the week.

One of "EveryDay", "WeekDays", "WeekEnd"

Or...

List of weekdays ["Monday", "Tuesday", ...]

## set_points

List of set points with time of day and temperature

