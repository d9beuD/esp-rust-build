# ESP8266 rust-build

This repository contains:

- Workflows for building [d9beuD/esp-rust](https://github.com/d9beuD/esp-rust) with ESP8266 Xtensa support
- Public binary artifacts in [Releases](https://github.com/d9beuD/esp-rust-build/releases)


Install release `v1.98.0.0` with `bash install.sh v1.98.0.0` on macOS ARM64 or Linux x64, or `./install.ps1 v1.98.0.0` on Windows x64. Installers verify `SHA256SUMS`, link `esp8266`, and include `rust-src` in toolchain sysroot.

Release workflows use GitHub-hosted `macos-14` (ARM64), `ubuntu-latest` (x64), and `windows-2022` (x64) runners, with architecture guards before builds. Evidence records source commit and CI result for each host. Hardware claims belong only in recorded LoLin ESP8266 board-verifier evidence.

Release workflows build source ref `esp-1.98.0.0` separately from release tag `v1.98.0.0`. Every run uploads its artifacts for CI retrieval; release assets upload only when that release tag already exists.

`support/release-bundle.sh` assembles already-built host archives into a local release bundle, writes `SHA256SUMS`, and records every host as `not-run`. It does not validate non-native archives or claim CI success. Check documentation links with `python3 support/check-doc-links.py`.
