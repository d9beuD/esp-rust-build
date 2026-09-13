param([string]$Tag, [switch]$Help)

if ($Help -or -not $Tag) { Write-Output 'usage: install.ps1 <release-tag>'; exit 0 }
$ErrorActionPreference = 'Stop'
$version = $Tag.TrimStart('v')
$host = if ($IsWindows -and [Environment]::Is64BitOperatingSystem) { 'x86_64-pc-windows-msvc' } else { throw 'unsupported host' }
$base = "https://github.com/d9beuD/esp-rust-build/releases/download/$Tag"
$hostAsset = "rust-$version-$host.zip"
$srcAsset = "rust-src-$version.tar.xz"
$work = Join-Path ([IO.Path]::GetTempPath()) ([guid]::NewGuid())
New-Item -ItemType Directory -Path $work | Out-Null
try {
  foreach ($asset in @($hostAsset, $srcAsset, 'SHA256SUMS')) { Invoke-WebRequest -Uri "$base/$asset" -OutFile (Join-Path $work $asset) }
  $sums = Get-Content (Join-Path $work 'SHA256SUMS')
  foreach ($asset in @($hostAsset, $srcAsset)) {
    $expected = ($sums | Where-Object { $_ -match [regex]::Escape($asset) } | Select-Object -First 1).Split()[0]
    if ((Get-FileHash (Join-Path $work $asset) -Algorithm SHA256).Hash.ToLower() -ne $expected.ToLower()) { throw "checksum failed: $asset" }
  }
  Expand-Archive (Join-Path $work $hostAsset) -DestinationPath $work
  tar -xJf (Join-Path $work $srcAsset) -C $work
  $toolchain = Join-Path $work "rust-$version-$host"
  New-Item -ItemType Directory -Force -Path (Join-Path $toolchain 'lib/rustlib/src') | Out-Null
  Move-Item (Join-Path $work "rust-src-$version/rust-src/lib/rustlib/src/rust") (Join-Path $toolchain 'lib/rustlib/src/rust')
  rustup toolchain link esp8266 $toolchain
  rustup run esp8266 rustc --print target-list | Select-String -SimpleMatch 'xtensa-esp8266-none-elf'
} finally { Remove-Item -Recurse -Force $work }
