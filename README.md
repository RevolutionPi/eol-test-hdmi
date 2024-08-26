<!--
SPDX-FileCopyrightText: 2023 KUNBUS GmbH

SPDX-License-Identifier: GPL-2.0-or-later
-->

# eol-test-hdmi

Show red, green, and blue colours on the framebuffer at `/dev/fb0` for 1 second
each, and play 3 sine waves at different frequencies over the default ALSA
device, each 1 second long. These tests run in parallel.

## Compiling

### Dependencies

Get the dependencies either through your system's package manager or through
[rustup](https://rustup.rs/) (recommended).

- `rustc`
- `cargo`
- `libasound` (Debian: `libasound2-dev`)

### Configuration

The `PKG_CONFIG_SYSROOT_DIR` environment variable should be set to point to the
directory that stores the necessary library headers.
Similarly, the linker should be set to use a target-specific one.

An example `.cargo/config.toml` to build for `armhf` on Debian could look like
this:

```toml
[build]
target = "armv7-unknown-linux-gnueabihf"

[target.armv7-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"

[env]
PKG_CONFIG_SYSROOT_DIR="/usr/arm-linux-gnueabihf"
```

Or like this for `aarch64`:

```toml
[build]
target = "aarch64-unknown-linux-gnu"

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"

[env]
PKG_CONFIG_SYSROOT_DIR="/usr/aarch64-linux-gnu"
```

It's important that, when cross-compiling, the `rust-std` component is
installed for the target architecture. If this is not the case the compiler
will output a long list of errors that point to the standard library missing
for the target.

Ideally, `rust-toolchain.toml` should also define the `targets` for which the
application will be compiled for. For example, to have both the `armv7` and
`aarch64` targets installed:

```toml
[toolchain]
# ...
targets = ["armv7-unknown-linux-gnueabihf", "aarch64-unknown-linux-gnu"]
# ...
```

A full example for a `rust-toolchain.toml` file that uses Rust version 1.63.0
and compiles for `aarch64-unknown-linux-gnu`, with the required components
installed, may look like this:

```toml
[toolchain]
channel = "1.63.0"
targets = ["aarch64-unknown-linux-gnu"]
components = ["rustc", "cargo", "rust-std", "clippy"]
```

### Cargo

Compile the project with `cargo build --release`. You can specify an
alternative target with the `target` flag.
For example, to compile for `armhf`:

```sh
cargo build --release --target armv7-unknown-linux-gnueabihf
```

Alternatively, the target can be set in the `.cargo/config.toml` file.

### Docker

Docker can be used to cross-compile the application for a different target and
glibc version. For this, the `Dockerfile` in this repository can be used.  
It has cross-compilation set up for aarch64/amd64 and armv7/armhf. To only use
one of the 2 comment the other one out, both in the section where the toolchain
is being installed with `rustup` and the dependencies that are installed.

For cross-compilation, a `.cargo/config.toml` should be present specifying all
the options needed. See section `Configuration` above.

## Running

After compiling, the binary will be in
`target/<host-triple>/release/eol-test-hdmi`. Alternatively it can be run with

```sh
cargo run --release
```

which will compile and run the project.
