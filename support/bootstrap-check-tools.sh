#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
tools="$root/.tools/checks"
bin="$tools/bin"

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) ;;
  *) printf 'unsupported host: %s-%s\n' "$(uname -s)" "$(uname -m)" >&2; exit 1 ;;
esac

download() {
  local url=$1 sha256=$2 archive=$3
  curl -fsSL "$url" -o "$tools/$archive"
  printf '%s  %s\n' "$sha256" "$tools/$archive" | shasum -a 256 -c -
}

mkdir -p "$tools" "$bin"
if [ ! -x "$bin/actionlint" ]; then
  download 'https://github.com/rhysd/actionlint/releases/download/v1.7.7/actionlint_1.7.7_darwin_arm64.tar.gz' '2693315b9093aeacb4ebd91a993fea54fc215057bf0da2659056b4bc033873db' actionlint.tar.gz
  tar -xzf "$tools/actionlint.tar.gz" -C "$bin" actionlint
fi
if [ ! -x "$bin/shellcheck" ]; then
  download 'https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.darwin.aarch64.tar.xz' '56affdd8de5527894dca6dc3d7e0a99a873b0f004d7aabc30ae407d3f48b0a79' shellcheck.tar.xz
  tar -xJf "$tools/shellcheck.tar.xz" -C "$tools"
  cp "$tools/shellcheck-v0.11.0/shellcheck" "$bin/shellcheck"
fi
if [ ! -x "$bin/pwsh" ]; then
  download 'https://github.com/PowerShell/PowerShell/releases/download/v7.5.1/powershell-7.5.1-osx-arm64.tar.gz' 'd1f016ccce5a7106e36090bf13ae71b46115f256465aee07760b26e607f4d033' powershell.tar.gz
  mkdir -p "$tools/powershell"
  tar -xzf "$tools/powershell.tar.gz" -C "$tools/powershell"
  chmod +x "$tools/powershell/pwsh"
  ln -sf ../powershell/pwsh "$bin/pwsh"
fi
printf '%s\n' "$bin"
