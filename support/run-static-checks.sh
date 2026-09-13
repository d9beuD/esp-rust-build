#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
bin=$("$root/support/bootstrap-check-tools.sh")
cd "$root"

"$bin/actionlint" .github/workflows/*.y*ml
"$bin/shellcheck" "$root"/*.sh "$root"/support/*.sh "$root"/support/rust-build/*/*.sh "$root"/../esp-rust/esp8266-poc/*.sh "$root"/../esp-rust/esp8266-poc/tests/*.sh
# shellcheck disable=SC2016
CHECK_ROOT="$root" "$bin/pwsh" -NoProfile -Command '
  $errors = @()
  Get-ChildItem -Path $env:CHECK_ROOT -Recurse -Filter *.ps1 | ForEach-Object {
    [System.Management.Automation.Language.Parser]::ParseFile($_.FullName, [ref]$null, [ref]$errors) | Out-Null
  }
  if ($errors.Count) { $errors | ForEach-Object { $_.ToString() }; exit 1 }
'
