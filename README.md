This project aims to provide an alternate UI for a rooted Gen 2 Nest Thermostat.

Highlevel list of things I'd like to achieve:
* Speak directly to home assistant, without extra services
   * esphome API: Can I use this? Seems like a good fit. I like how esphome
     devices magically just "show up" in home assistant
   * REST API: Maybe this makes more sense, and still acheives the same result?
   * MQTT: If the above options don't pan out, it looks like using MQTT would
     be pretty straight forward. I just don't love having to run the MQTT
     server container if I don't have to.
* Some level of power management
   * Screen off after timeout, wake on press
   * Does the stock app do anything special with power state?
   * Device goes into some sort of "deep sleep" when not connected to USB or
     backplate. If it still goes into this sleep state with stock app stopped,
     it must be handled already at a lower level.
* Main thermostat inteface
   * Functionally similar to stock, except:
     - Scheduling on device. I always found this awkward.
       Seems like scheduling in home assistant would be a better UX.
   * Same dial interface for changing target temp
   * Target temp, current temp, is furnace or AC running?
   * Select between heating, cooling, fan, off
* Extensible list of screens to display when pressing thermostat
   * Full configurable screens? Or maybe a few that are always in the list?
     (e.g. heating/cooling/fan/off select)
   * Example: Screen that displays a list of basic switches in home assistant.


## Requirements

Add target with `rustup`

```bash
rustup target add armv7-unknown-linux-gnueabihf
```

Get toolchain from https://github.com/eckucukoglu/arm-linux-gnueabihf
and add to `PATH`.

```bash
export PATH=~/Toolchains/arm-linux-gnueabihf/bin:$PATH
```

Create `~/.cargo/config.toml` with linker command from toolchain.

```toml
[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```

## Building & Running

First; the stock app needs to be stopped.

```bash
/etc/init.d/nestlabs stop
```

Optional: start `nest-app.sh` on device and use `build_push.sh` to build,
push to device, and restart app (uses netcat on port 51234).

```bash
./build_push.sh
```

Or build and send manually.

```bash
# Build output at `target/armv7-unknown-linux-gnueabihf/release/nest-app`
cargo build --target=armv7-unknown-linux-gnueabihf
```