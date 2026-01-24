Implementation of the [ESPHome Native API](https://esphome.io/components/api/)
for acting as an ESPHome device.

Why not use? https://crates.io/crates/esphome-native-api

That looks like a great option. But it uses async. I would have used this,
but I'm avoiding async in favour of standard Rust threads.

I've implemented it syncronsouly with the hope that the consumer can decide what
the threading model should be. However, I suspect in its current state, it would
not as efficient as possible (eg. io operations are currently sync).

The API version currently used is 2025.12.2, which can be changed by fetching
`.proto` files from the desired version tag:

https://github.com/esphome/esphome/tree/2025.12.2/esphome/components/api

Then adjust the necessary paths in [build.rs](build.rs)
