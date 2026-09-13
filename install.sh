#!/usr/bin/env bash
set -euo pipefail

usage() { printf 'usage: %s <release-tag>\n' "$0"; }
[ "${1:-}" = --help ] || [ "${1:-}" = -h ] && { usage; exit 0; }
tag=${1:?"$(usage)"}
version=${tag#v}
case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) host=aarch64-apple-darwin ;;
  Linux-x86_64) host=x86_64-unknown-linux-gnu ;;
  *) printf 'unsupported host: %s-%s\n' "$(uname -s)" "$(uname -m)" >&2; exit 1 ;;
esac

# Public default endpoint: https://github.com/d9beuD/esp-rust-build/releases/download/v1.98.0.0/SHA256SUMS
base="https://github.com/d9beuD/esp-rust-build/releases/download/$tag"
host_asset="rust-$version-$host.tar.xz"
src_asset="rust-src-$version.tar.xz"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
cd "$work"
for asset in "$host_asset" "$src_asset" SHA256SUMS; do
  curl -fsSL "$base/$asset" -o "$asset"
done
if command -v sha256sum >/dev/null; then sha256sum -c SHA256SUMS; else shasum -a 256 -c SHA256SUMS; fi
tar -xJf "$host_asset"
tar -xJf "$src_asset"
toolchain_dir="$work/rust-$version-$host"
mkdir -p "$toolchain_dir/lib/rustlib/src"
mv "$work/rust-src-$version/rust-src/lib/rustlib/src/rust" "$toolchain_dir/lib/rustlib/src/rust"
rustup toolchain link esp8266 "$toolchain_dir"
rustup run esp8266 rustc --print target-list | grep -Fx xtensa-esp8266-none-elf
