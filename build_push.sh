#!/bin/bash

if cargo build --no-default-features --features "device" --target=armv7-unknown-linux-gnueabihf --release; then
   echo "Sending build via netcat"
   cat target/armv7-unknown-linux-gnueabihf/release/nest-app | nc -q0 nest-dev 51234
fi
