#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf 'usage: %s <version> <source-commit> <macos-archive> <linux-archive> <windows-archive> <rust-src-archive> [output-dir]\n' "$0"
}

if [ "$#" -lt 6 ] || [ "$#" -gt 7 ]; then
  usage >&2
  exit 64
fi
version=$1
commit=$2
macos=$3
linux=$4
windows=$5
rust_src=$6
output=${7:-.}

if [[ ! $commit =~ ^[0-9a-f]{40}$ ]]; then
  printf 'source commit must be 40 lowercase hexadecimal characters\n' >&2
  exit 64
fi

mkdir -p "$output"
copy_asset() {
  local source=$1 destination=$2
  [ -f "$source" ] || { printf 'missing asset: %s\n' "$source" >&2; exit 66; }
  if [ "$source" != "$output/$destination" ]; then cp "$source" "$output/$destination"; fi
}

copy_asset "$macos" "rust-$version-aarch64-apple-darwin.tar.xz"
copy_asset "$linux" "rust-$version-x86_64-unknown-linux-gnu.tar.xz"
copy_asset "$windows" "rust-$version-x86_64-pc-windows-msvc.zip"
copy_asset "$rust_src" "rust-src-$version.tar.xz"

(
  cd "$output"
  sha256sum "rust-$version-aarch64-apple-darwin.tar.xz" "rust-$version-x86_64-unknown-linux-gnu.tar.xz" "rust-$version-x86_64-pc-windows-msvc.zip" "rust-src-$version.tar.xz" > SHA256SUMS
  printf '{"tag":"v%s","version":"%s","source":{"repository":"https://github.com/d9beuD/esp-rust.git","commit":"%s"},"hosts":[{"host":"aarch64-apple-darwin","ci":"not-run"},{"host":"x86_64-unknown-linux-gnu","ci":"not-run"},{"host":"x86_64-pc-windows-msvc","ci":"not-run"}]}\n' "$version" "$version" "$commit" > evidence.json
)
