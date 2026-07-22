#!/bin/bash
# set -e

# build script
export ZIG_GLOBAL_CACHE_DIR=$PWD/target/zig-cache # [src](https://github.com/ziglang/zig/issues/19400) - global cache in home by default for Zig

cargo zigbuild --release --target aarch64-unknown-linux-gnu
ssh hamdan@rasso.local "killall oodssistant"
scp target/aarch64-unknown-linux-gnu/release/oodssistant hamdan@rasso.local:~
ssh hamdan@rasso.local "~/oodssistant"