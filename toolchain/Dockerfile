FROM ghcr.io/jiggak/nest-toolchain AS nest-toolchain

FROM rust:1.95.0

COPY --from=nest-toolchain /arm-nest-linux-gnueabihf /arm-nest-linux-gnueabihf

RUN rustup target add armv7-unknown-linux-gnueabihf

# clang for bingen crate
# protoc for esphome-api crate
RUN apt-get update && apt-get install -y \
   libclang-dev \
   protobuf-compiler

ENV PATH="/arm-nest-linux-gnueabihf/bin:${PATH}"

RUN mkdir /work
WORKDIR /work
