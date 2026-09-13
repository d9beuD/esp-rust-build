use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const VERSION: &str = "1.98.0.0";
const TAG: &str = "v1.98.0.0";
const FORK: &str = "https://github.com/d9beuD/esp-rust.git";

fn build_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tests directory must be inside esp-rust-build")
        .to_path_buf()
}

fn repo_root() -> PathBuf {
    build_root()
        .parent()
        .expect("esp-rust-build must be inside repository")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[cfg(unix)]
fn write_executable(path: &Path, contents: &str) {
    use std::os::unix::fs::PermissionsExt;

    fs::write(path, contents).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn host_workflows() -> [(&'static str, &'static str); 3] {
    [
        ("build-aarch64-apple-darwin.yaml", "aarch64-apple-darwin"),
        ("build-x86_64-unknown-linux-gnu.yaml", "x86_64-unknown-linux-gnu"),
        ("build-x86_64-pc-windows-msvc.yaml", "x86_64-pc-windows-msvc"),
    ]
}

#[test]
fn public_release_checksum_download_is_unauthenticated() {
    let installer = build_root().join("install.sh");
    assert!(installer.is_file(), "POSIX installer missing: {}", installer.display());

    // install.sh must use this public endpoint. Its curl process is replaced in
    // installer tests, so this test never contacts GitHub or relies on a release.
    let source = read(&installer);
    let expected = format!(
        "https://github.com/d9beuD/esp-rust-build/releases/download/{TAG}/SHA256SUMS"
    );
    assert!(source.contains(&expected), "installer must download {expected}");
    assert!(
        !source.contains("Authorization:") && !source.contains("GITHUB_TOKEN"),
        "public release download must not send GitHub credentials"
    );
}

#[test]
fn every_release_workflow_publishes_named_stage2_source_manifest_and_evidence() {
    let workflows = build_root().join(".github/workflows");
    let mut missing = Vec::new();
    for (workflow, host) in host_workflows() {
        let source = read(&workflows.join(workflow));
        for asset in [
            format!("rust-${{{{ github.event.inputs.release_version }}}}-{host}.{}", if host.contains("windows") { "zip" } else { "tar.xz" }),
            "rust-src-${{ github.event.inputs.release_version }}.tar.xz".to_owned(),
            "SHA256SUMS".to_owned(),
            "evidence.json".to_owned(),
        ] {
            if !source.contains(&asset) {
                missing.push(format!("{workflow} does not publish {asset}"));
            }
        }
        if source.contains("continue-on-error: true") || source.contains("|| echo") {
            missing.push(format!("{workflow} allows a dist build failure"));
        }
    }
    assert!(missing.is_empty(), "{}", missing.join("; "));
}

#[test]
fn release_workflows_use_explicit_source_refs_and_optional_release_uploads() {
    let workflows = build_root().join(".github/workflows");
    let mut violations = Vec::new();
    for workflow in [
        "build-aarch64-apple-darwin.yaml",
        "build-x86_64-unknown-linux-gnu.yaml",
        "build-x86_64-pc-windows-msvc.yaml",
        "build-rust-src.yaml",
    ] {
        let source = read(&workflows.join(workflow));
        for required in [
            "source_ref:",
            "required: true",
            "repository: d9beuD/esp-rust",
            "ref: ${{ github.event.inputs.source_ref }}",
            "evidence.json",
            FORK,
        ] {
            if !source.contains(required) {
                violations.push(format!("{workflow} missing {required}"));
            }
        }
        let source_ref_input = source
            .split("source_ref:")
            .nth(1)
            .and_then(|input| input.split("release_tag:").next());
        if source_ref_input.is_none_or(|input| input.contains("default:")) {
            violations.push(format!("{workflow} does not require an explicit source_ref"));
        }
        if !source.contains("rev-parse HEAD") {
            violations.push(format!("{workflow} does not resolve a 40-character source commit"));
        }

        if !source.contains("release_tag:")
            || !source.contains(&format!("default: \"v{VERSION}\""))
        {
            violations.push(format!("{workflow} does not retain a v-prefixed release_tag"));
        }

        let run_artifact_step = source
            .split("- name: Upload run artifacts")
            .nth(1)
            .and_then(|step| step.split("\n      - ").next());
        match run_artifact_step {
            Some(step)
                if step.contains("uses: actions/upload-artifact@v4")
                    && !step.contains("needs.get_release.outputs.upload_url") => {}
            _ => violations.push(format!("{workflow} does not upload run artifacts without a release")),
        }

        for step in source
            .split("\n      - ")
            .filter(|step| step.contains("uses: actions/upload-release-asset@v1"))
        {
            if !step.contains("if: needs.get_release.outputs.upload_url != ''") {
                violations.push(format!("{workflow} uploads a release asset without an existing release"));
            }
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("; "));
}

#[test]
fn release_workflows_generate_manifest_that_rejects_mutated_assets() {
    let workflows = build_root().join(".github/workflows");
    let mut violations = Vec::new();
    for (workflow, _) in host_workflows() {
        let source = read(&workflows.join(workflow));
        if !source.contains("sha256sum") {
            violations.push(format!("{workflow} does not generate SHA-256 checksums"));
        }
        if !source.contains("SHA256SUMS") {
            violations.push(format!("{workflow} does not upload SHA256SUMS"));
        }
    }
    assert!(violations.is_empty(), "{}", violations.join("; "));
}

#[test]
fn host_packaging_discovers_one_stage2_output_before_renaming_release_assets() {
    let workflows = build_root().join(".github/workflows");
    for (workflow, host) in host_workflows() {
        let source = read(&workflows.join(workflow));
        if host.contains("windows") {
            assert!(source.contains("source_archives=(rust/build/dist/rust-src-*.tar.xz)"));
            assert!(source.contains("[ \"${#source_archives[@]}\" -ne 1 ]"));
        } else {
            let archive = format!("archives=(build/dist/rust-*-{host}.tar.xz)");
            assert!(source.contains(&archive), "{workflow} must discover its stage2 host archive");
            assert!(source.contains("[ \"${#archives[@]}\" -ne 1 ]"));
            assert!(source.contains("mv \"${archives[0]}\" \"rust-${{ github.event.inputs.release_version }}"));
        }
    }

    let repackage = read(&build_root().join("support/rust-build/Repackage-RustRelease.ps1"));
    for required in [
        "Get-ChildItem -File -Filter \"rust-*-${DefaultHost}.tar.xz\"",
        "$RustArchives.Count -ne 1",
        "Get-ChildItem -Directory -Filter \"rust-*-${DefaultHost}\"",
        "$RustDirectories.Count -ne 1",
    ] {
        assert!(repackage.contains(required), "Windows packaging must contain {required}");
    }
    let workflow = read(&workflows.join("build-x86_64-pc-windows-msvc.yaml"));
    assert!(workflow.contains("-ReleaseVersion \"${{ github.event.inputs.release_version }}\""));
}

#[test]
fn rust_src_workflow_renames_dist_archive_before_provenance_and_checksum() {
    let source = read(&build_root().join(".github/workflows/build-rust-src.yaml"));
    let archive = "rust-src-${{ github.event.inputs.release_version }}.tar.xz";
    let rename = format!("cp build/dist/rust-src-nightly.tar.xz \"../{archive}\"");
    let provenance = "printf '{\"tag\":\"%s\"";
    let checksum = format!("sha256sum \"../{archive}\" > ../SHA256SUMS");

    let renamed_at = source.find(&rename).expect("rust-src dist archive must be renamed for release");
    let provenance_at = source.find(provenance).expect("rust-src provenance must be recorded");
    let checksum_at = source.find(&checksum).expect("renamed rust-src archive must be checksummed");
    assert!(renamed_at < provenance_at && provenance_at < checksum_at);
}

#[test]
#[cfg(unix)]
fn posix_installer_verifies_downloads_links_toolchain_and_exposes_builtin_target() {
    let installer = build_root().join("install.sh");
    assert!(installer.is_file(), "POSIX installer missing: {}", installer.display());

    let fixture = std::env::temp_dir().join(format!("esp8266-installer-contract-{}", std::process::id()));
    let assets = fixture.join("assets");
    let bin = fixture.join("bin");
    let unpacked = fixture.join("unpacked");
    fs::create_dir_all(&assets).unwrap();
    fs::create_dir_all(&bin).unwrap();
    let host = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        other => panic!("no POSIX release fixture for {}-{}", other.0, other.1),
    };
    let toolchain_dir = format!("rust-{VERSION}-{host}");
    fs::create_dir_all(unpacked.join(&toolchain_dir).join("bin")).unwrap();
    fs::create_dir_all(unpacked.join("rust-src-1.98.0.0/rust-src/lib/rustlib/src/rust/library/core")).unwrap();
    fs::write(unpacked.join(&toolchain_dir).join("bin/rustc"), "toolchain").unwrap();
    fs::write(unpacked.join("rust-src-1.98.0.0/rust-src/lib/rustlib/src/rust/library/core/lib.rs"), "core").unwrap();
    let host_asset = assets.join(format!("rust-{VERSION}-{host}.tar.xz"));
    let src_asset = assets.join("rust-src-1.98.0.0.tar.xz");
    for (archive, member) in [(&host_asset, toolchain_dir.as_str()), (&src_asset, "rust-src-1.98.0.0")] {
        let status = Command::new("tar")
            .args(["-C", unpacked.to_str().unwrap(), "-cJf", archive.to_str().unwrap(), member])
            .status()
            .expect("tar must be available for installer fixture");
        assert!(status.success(), "fixture archive creation failed");
    }
    let sums = Command::new("shasum")
        .args(["-a", "256", host_asset.to_str().unwrap(), src_asset.to_str().unwrap()])
        .output()
        .expect("shasum must be available for installer fixture");
    assert!(sums.status.success());
    fs::write(assets.join("SHA256SUMS"), sums.stdout).unwrap();
    write_executable(&bin.join("curl"), "#!/usr/bin/env bash\nset -euo pipefail\nfor value in \"$@\"; do [[ $value == http* ]] && url=$value; done\n[[ ${url:?} == https://github.com/d9beuD/esp-rust-build/releases/download/v1.98.0.0/* ]]\n[[ \" $* \" != *Authorization* ]]\ncp \"$FIXTURE_ASSETS/${url##*/}\" \"${@: -1}\"\n");
    write_executable(&bin.join("rustup"), "#!/usr/bin/env bash\nset -euo pipefail\nif [[ $1 == toolchain && $2 == link && $3 == esp8266 ]]; then\n  test -f \"$4/bin/rustc\"\n  test -f \"$4/lib/rustlib/src/rust/library/core/lib.rs\"\n  printf '%s' \"$4\" > \"$RUSTUP_HOME/linked\"\nelif [[ $1 == run && $2 == esp8266 && $3 == rustc && $4 == --print && $5 == target-list ]]; then\n  test -f \"$RUSTUP_HOME/linked\"\n  printf '%s\\n' xtensa-esp8266-none-elf\nelse\n  exit 64\nfi\n");
    let rustup_home = fixture.join("rustup");
    fs::create_dir_all(&rustup_home).unwrap();
    let output = Command::new("bash")
        .arg(&installer)
        .arg(TAG)
        .env("FIXTURE_ASSETS", &assets)
        .env("RUSTUP_HOME", &rustup_home)
        .env("PATH", format!("{}:{}", bin.display(), std::env::var("PATH").unwrap()))
        .output()
        .expect("bash must execute POSIX installer");
    assert!(output.status.success(), "installer failed: {}", String::from_utf8_lossy(&output.stderr));
    let target_list = Command::new(bin.join("rustup"))
        .args(["run", "esp8266", "rustc", "--print", "target-list"])
        .env("RUSTUP_HOME", &rustup_home)
        .output()
        .unwrap();
    assert!(target_list.status.success());
    assert_eq!(String::from_utf8(target_list.stdout).unwrap(), "xtensa-esp8266-none-elf\n");
    fs::remove_dir_all(fixture).unwrap();
}

#[test]
fn powershell_installer_verifies_downloads_links_toolchain_and_exposes_builtin_target() {
    let installer = build_root().join("install.ps1");
    assert!(installer.is_file(), "PowerShell installer missing: {}", installer.display());

    let source = read(&installer);
    for required in ["SHA256SUMS", "Get-FileHash", "rustup toolchain link esp8266", "rust-src"] {
        assert!(source.contains(required), "installer must contain {required}");
    }
}

#[test]
fn clean_host_matrix_installs_and_builds_example_on_all_release_hosts() {
    let workflow = read(&repo_root().join("esp-rust/.github/workflows/esp8266-poc.yaml"));
    for required in ["macos", "ubuntu", "windows", "install.sh", "install.ps1", "cargo build --release"] {
        assert!(workflow.contains(required), "ESP8266 POC CI must contain {required}");
    }
}

#[test]
fn cargo_poc_declares_no_std_lx106_build_wiring_and_builds_release_firmware() {
    let poc = repo_root().join("esp-rust/esp8266-poc");
    let manifest = poc.join("Cargo.toml");
    assert!(manifest.is_file(), "Cargo POC manifest missing: {}", manifest.display());
    let config = poc.join(".cargo/config.toml");
    assert!(config.is_file(), "Cargo POC config missing: {}", config.display());

    let manifest_source = read(&manifest);
    let config_source = read(&config);
    let main_source = read(&poc.join("src/main.rs"));
    assert!(main_source.contains("#![no_std]"), "firmware must remain no_std");
    for required in ["xtensa-esp8266-none-elf", "xtensa-lx106-elf-gcc", "build-std", "link.x", "start.S"] {
        assert!(
            manifest_source.contains(required) || config_source.contains(required),
            "Cargo wiring must contain {required}"
        );
    }

    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&poc)
        .status()
        .expect("cargo must be available to execute POC release build");
    assert!(status.success(), "cargo build --release must succeed with installed esp8266 toolchain and LX106 GCC");
}

#[test]
fn docs_limit_lolin_hardware_claim_to_recorded_successful_board_evidence() {
    let readme = repo_root().join("esp-rust/esp8266-poc/README.md");
    assert!(readme.is_file(), "ESP8266 POC evidence documentation missing: {}", readme.display());
    let source = read(&readme);
    assert!(source.contains("LoLin ESP8266"), "docs must name only supported LoLin ESP8266 hardware");
    assert!(source.contains("verify-board.sh --port <port> --led-observed"), "docs must require recorded board verifier command");
    assert!(source.contains("flash=PASS readback=PASS static=PASS serial=PASS timing=PASS led=PASS"), "docs must record complete successful board evidence");
}
