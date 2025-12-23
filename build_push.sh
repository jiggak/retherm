#!/bin/bash

if cargo build --target=armv7-unknown-linux-gnueabihf --release; then
   echo "Sending build via netcat"
   cat target/armv7-unknown-linux-gnueabihf/release/nest-app | nc -w1 nest-dev 51234
fi
