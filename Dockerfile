# Compile a given source for aarch64/amd64 or armv7/armhf.
# If only one of the two is needed the other can simply be commented out.
# The target that should be compiled should be set in the project's
# `.cargo/config.toml` file in the `build` section using the `target` attribute.

FROM rust:1-bullseye

ENV RUST_COMPONENTS="rustc,cargo,rust-std,clippy"

RUN set -eux; \
    rustup toolchain install stable-aarch64-unknown-linux-gnu \
        --component "${RUST_COMPONENTS}" \
    && rustup toolchain install stable-armv7-unknown-linux-gnueabihf \
        --component "${RUST_COMPONENTS}"

# dependencies for compiling
RUN set -eux; \
    dpkg --add-architecture arm64 \
    && dpkg --add-architecture armhf \
    && apt-get update \
    && apt-get install -y \
        build-essential \
        pkg-config \
        gcc-aarch64-linux-gnu \
        gcc-arm-linux-gnueabihf \
        # TODO: amd64 deps shouldn't be needed for cross compiling
        libasound2 \
        libasound2-dev \
        libasound2:arm64 \
        libasound2-dev:arm64 \
        libasound2:armhf \
        libasound2-dev:armhf

CMD ["cargo", "build", "--release"]
