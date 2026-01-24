# (Re)Therm

(Re)store remote management of Gen2 Nest Thermostat through your existing
Home Assistant setup.

This project aims to provide a completely new UI for a rooted Gen 2 Nest
Thermostat.

## Highlevel list of features for MVP

- [x] Bi-directional Home Assistant connection using esphome API
- [ ] Automatic device discovery in Home Assistant (mDNS)
- [x] Dial interface similar to stock Nest UI with mode select
- [ ] Turning the HVAC system on/off to reach target temp.
- [ ] Configurable interface look/feel (separate from app config)
- [ ] Configuration file for settings such as:
  - [ ] HA related parameters (api key, device name, etc)
  - [ ] Wifi network settings
  - [ ] Screen brightness, auto-off timeout
  - [ ] HVAC wiring settings
- [ ] Integrate with system wifi manager (Connman 1.29)
- [x] Screen auto-off, wake on user input

## Build Requirements

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

Optional: start `build_recv.sh` on device and use `build_push.sh` to build,
push to device, and restart app (uses netcat on port 51234).

```bash
./build_push.sh
```

Or build and send manually.

```bash
# Build output at `target/armv7-unknown-linux-gnueabihf/release/retherm`
cargo build --no-default-features --features device --target=armv7-unknown-linux-gnueabihf
```

## Stretch goals

- [ ] Fancy animations for screen transtions
- [ ] Extensible list of screens through configuration
  - Possible use case would be controlling other HA devices through
    some sort of configurable menu system
  - Example: Screen for turning on/off other devices

## What about power management?

I'm not sure how important this is since the Nest is always powered.
But I can see how getting the most out of the battery in the event of a power
outage could be important.

This is the behaviour I've observed with the stock Nest app.

* I'm assuming that when the device can no longer be pinged, it has gone into
  some sort of sleep mode
* Waking the screen screen causes network reconnect
* With the display connected to USB for power, it _usually_ remains network
  accessible for some time after the screen turns off, but it will eventually
  stop responding to ping
* With display disconnected from power, it disconnects from network soon after
  display turns off (I have not timed how long it remains accessible)
  * However, if you happen to open an SSH session befor the screen turns off,
    the session will remain active for a while (few minutes at most)
  * SSH session hangs, it doesn't disconnnect
  * If you are quick about it, you can wake the display and resume a hung session
  * If it hangs too long, waking device doesn't help, session remains hung
* When the stock app is **stopped**, and the display is **not** connected to
  power, the device seems to remain network accessible for much longer
  (presummably until battery dies)
* Need to look into what Nest app does; could it be as simple setting kernel
  power state?

