#!/bin/bash

HOST=nest-dev

EXAMPLE=$1
if [ "${EXAMPLE}" != "" ]; then
   ARGS="--example ${EXAMPLE}"
   OUTPUT=target/armv7-unknown-linux-gnueabihf/examples/${EXAMPLE}
else
   ARGS="--no-default-features --features device"
   OUTPUT=target/armv7-unknown-linux-gnueabihf/release/retherm
fi

if cargo build ${ARGS} --target=armv7-unknown-linux-gnueabihf --release; then
   echo "Sending build via netcat"
   cat ${OUTPUT} | nc -q0 ${HOST} 51234
fi
