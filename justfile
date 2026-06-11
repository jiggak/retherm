set quiet

export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER := "arm-nest-linux-gnueabihf-gcc"
TOOLCHAIN_IMAGE_NAME := "retherm-toolchain"

export PATH := x".toolchain/arm-nest-linux-gnueabihf/bin:${PATH}"

# Build with host toolchain
build:
    cargo build --no-default-features --features device --target=armv7-unknown-linux-gnueabihf --release

# Build with docker toolchain image
build-docker: build-toolchain-image
    docker run --rm -t --user $(id -u):$(id -g) \
       -e CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER \
       -v "$(pwd):/work" {{TOOLCHAIN_IMAGE_NAME}} \
       cargo build --no-default-features --features device --target=armv7-unknown-linux-gnueabihf --release

# Push build to nest
push host=env("NEST_HOST", "nest-dev"): build
    echo "Sending to {{host}} with netcat"
    cat target/armv7-unknown-linux-gnueabihf/release/retherm | nc -q0 {{host}} 51234

# Build toolchain docker image
[working-directory: "toolchain"]
build-toolchain-image:
    docker build . -t {{TOOLCHAIN_IMAGE_NAME}}

# Copy arm toolchain from docker image to host
get-toolchain:
    mkdir -p .toolchain
    docker run --rm --user $(id -u):$(id -g) \
       -v ".toolchain:/output" \
       ghcr.io/jiggak/nest-toolchain \
       bash -c "cp -r /arm-nest-linux-gnueabihf /output/"
    echo "Toolchain copied to .toolchain/arm-nest-linux-gnueabihf"

# (Re)Generate website docs from struct comments
[env("RUSTDOCFLAGS", "-Z unstable-options --output-format json")]
[working-directory: "website"]
struct-markdown:
    cat content/configuration.tmpl > content/configuration.md
    cat content/theme.tmpl > content/theme.md

    # At this time, json output is an unstable feature; requires nightly tools
    cargo +nightly doc --no-deps

    cargo run -p docgen ../target/doc/retherm.json \
       Config AwayConfig BackplateConfig HomeAssistantConfig BacklightConfig ScheduleConfig \
       >>content/configuration.md

    cargo run -p docgen ../target/doc/retherm.json \
       Theme MainScreenTheme GaugeStyle ModeSelectTheme ListStyle \
       >>content/theme.md
