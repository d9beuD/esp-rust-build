#!/usr/bin/env bash
set -euo pipefail

dist=$1
host=$2
version=$3
root="rust-$version-$host"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

shopt -s nullglob
for component in rustc rust-std cargo clippy rustfmt; do
  archives=("$dist/$component-"*"-$host.tar.xz")
  if [ "${#archives[@]}" -ne 1 ]; then
    printf 'expected one %s stage2 archive, found %s\n' "$component" "${#archives[@]}" >&2
    exit 1
  fi
  tar -xJf "${archives[0]}" -C "$work"
done

mkdir "$work/$root"
cp -R "$work"/rustc-*/rustc/. "$work/$root/"
cp -R "$work"/rust-std-*"-$host"/rust-std-"$host"/. "$work/$root/"
cp -R "$work"/cargo-*"-$host"/cargo/. "$work/$root/"
cp -R "$work"/clippy-*"-$host"/clippy-preview/. "$work/$root/"
cp -R "$work"/rustfmt-*"-$host"/rustfmt-preview/. "$work/$root/"
tar -C "$work" -cJf "$dist/$root.tar.xz" "$root"
