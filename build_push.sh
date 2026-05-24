#!/bin/bash

if [ -z "${NEST_HOST}" ]; then
   echo "Set NEST_HOST to hostname of device to receive build" >&2
   exit 1
fi

EXAMPLE=$1
if [ "${EXAMPLE}" != "" ]; then
   ARGS="--example ${EXAMPLE}"
   OUTPUT=target/armv7-unknown-linux-gnueabihf/release/examples/${EXAMPLE}
else
   ARGS="--no-default-features --features device"
   OUTPUT=target/armv7-unknown-linux-gnueabihf/release/retherm
fi

export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-nest-linux-gnueabihf-gcc

if cargo build ${ARGS} --target=armv7-unknown-linux-gnueabihf --release; then
   echo "Sending build via netcat"
   cat ${OUTPUT} | nc -q0 ${NEST_HOST} 51234
fi
