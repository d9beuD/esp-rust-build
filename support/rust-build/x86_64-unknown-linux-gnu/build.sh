#!/usr/bin/env bash

set -e

checkout=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/rust

if git -C "$checkout" rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    git -C "$checkout" remote set-url origin https://github.com/d9beuD/esp-rust.git
    git -C "$checkout" fetch --depth 1 origin "${SOURCE_REF:-esp-${RELEASE_VERSION}}"
    git -C "$checkout" checkout --detach FETCH_HEAD
    git -C "$checkout" submodule update --init --recursive --depth 1
else
    rm -rf "$checkout"
    git clone --recursive --depth 1 --shallow-submodules https://github.com/d9beuD/esp-rust.git "$checkout" -b "${SOURCE_REF:-esp-${RELEASE_VERSION}}"
fi
cd "$checkout"
# TODO
# this doesn't work in the docker container when the host is ARM64, it fails at the documentation stage, which can be skipped with `--disable-docs`
# however, at the time of writing this disables rustdoc tool creation, which we need in the toolchain. Until this is fixed, we can just build on an x86_64 host.
python3 src/bootstrap/configure.py --experimental-targets=Xtensa --release-channel=nightly --release-description="${RELEASE_VERSION}" --enable-extended --enable-cargo-native-static --tools=rustdoc,clippy,cargo,rustfmt,rust-analyzer-proc-macro-srv,src --dist-compression-formats='xz' --enable-lld --enable-profiler --host x86_64-unknown-linux-gnu --set build.rustfmt="$(command -v rustfmt)"
python3 x.py dist --stage 2 rust-src
