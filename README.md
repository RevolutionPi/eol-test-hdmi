# eol-test-hdmi

Show red, green, and blue colours on the framebuffer at `/dev/fb0` for 1 second
each, and play 2 sine waves at different frequencies over the default ALSA
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

### Cargo

Compile the project with `cargo build --release`. You can specify an
alternative target with the `target` flag.
For example, to compile for `armhf`:

```sh
cargo build --release --target armv7-unknown-linux-gnueabihf
```

Alternatively, the target can be set in the `.cargo/config.toml` file.

## Running

After compiling, the binary will be in
`target/<host-triple>/release/eol-test-hdmi`. Alternatively it can be run with

```sh
cargo run --release
```

which will compile and run the project.
