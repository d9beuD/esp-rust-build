[CmdletBinding()]
param (
    [Parameter()]
    [String]
    [ValidateSet("x86_64-pc-windows-msvc", "x86_64-pc-windows-gnu")]
    $DefaultHost = "x86_64-pc-windows-msvc",
    [Parameter()]
    [String]
    $ReleaseVersion = "1.98.0.0"
)
# Helper script to perform repackaging of Windows release

# Stop on error
$ErrorActionPreference = "Stop"

if (Test-Path -Path esp -PathType Container) {
    Remove-Item -Recurse -Force -Path esp
    rm *.tar
}

$RustArchives = @(Get-ChildItem -File -Filter "rust-*-${DefaultHost}.tar.xz")
if ($RustArchives.Count -ne 1) { throw "expected one stage2 host archive, found $($RustArchives.Count)" }
$RustArchive = $RustArchives[0]
$RustVersionHost = $RustArchive.BaseName
$RustVersion = $RustVersionHost.Substring(5, $RustVersionHost.Length - 5 - $DefaultHost.Length - 1)

mkdir esp
7z e $RustArchive.Name
7z x "${RustVersionHost}.tar"
$RustDirectories = @(Get-ChildItem -Directory -Filter "rust-*-${DefaultHost}")
if ($RustDirectories.Count -ne 1) { throw "expected one stage2 host directory, found $($RustDirectories.Count)" }
pushd $RustDirectories[0].Name
cp -Recurse .\rustc\bin ..\esp\
cp -Recurse .\rustc\lib ..\esp\
cp -Recurse .\rustc\libexec ..\esp\
cp -Recurse .\rustc\share ..\esp\
cp -ErrorAction SilentlyContinue -Recurse .\rust-std-${DefaultHost}\lib\* ..\esp\lib\
popd

7z e rust-src-${RustVersion}.tar.xz
7z x rust-src-${RustVersion}.tar
pushd rust-src-${RustVersion}
cp -ErrorAction SilentlyContinue -Recurse .\rust-src\lib\* ..\esp\lib\
popd

7z e cargo-${RustVersionHost}.tar.xz
7z x cargo-${RustVersionHost}.tar
pushd cargo-${RustVersionHost}
cp -ErrorAction SilentlyContinue -Recurse .\cargo\bin\* ..\esp\bin\
cp -ErrorAction SilentlyContinue -Recurse .\cargo\libexec ..\esp\
popd

7z e clippy-${RustVersionHost}.tar.xz
7z x clippy-${RustVersionHost}.tar
pushd clippy-${RustVersionHost}
cp -ErrorAction SilentlyContinue -Recurse .\clippy-preview\bin\* ..\esp\bin\
popd

7z e rustfmt-${RustVersionHost}.tar.xz
7z x rustfmt-${RustVersionHost}.tar
pushd rustfmt-${RustVersionHost}
cp -ErrorAction SilentlyContinue -Recurse .\rustfmt-preview\bin\* ..\esp\bin\
popd

# Clean up debug files
Get-ChildItem -Path .\ -Filter *.pdb -Recurse -File -Name| ForEach-Object {
    "Removing: $_"
    Remove-Item -Path $_
}

7z a rust-${ReleaseVersion}-${DefaultHost}.zip esp/
