//! End-to-end tests for `vx check` on a numeric rust pin with no `rust-toolchain` file.
//!
//! These cover the exact reproduction from the issue: a repository declaring
//! `rust = "1.93.1"` and providing no toolchain file used to get no warning at all,
//! and `vx check` still exited 0.
//!
//! Contract under test:
//!
//! * declared != actual → a visible signal on stderr, and `vx check` exits 0 by
//!   default (warn is the non-breaking default);
//! * with `[check] toolchain_pin_mismatch = "error"` → `vx check` exits non-zero;
//! * declared == actual → no mismatch message at all;
//! * `[check] toolchain_pin_mismatch` also governs the **run** path (`vx cargo …`),
//!   which warns from the resolver rather than from `vx check`. `"ignore"` must
//!   silence it there too, while `"error"` must still only fail `vx check` — failing
//!   a build would leave an already-drifting repository unbuildable.

#![allow(clippy::unwrap_used)]

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Mutex;
use std::time::Duration;

use common::{assert_success, run_command_with_timeout, stderr_str, stdout_str};
use rstest::rstest;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(300);

/// Serializes tests that inspect the process-wide Rust environment.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Keep every fixture's writable state under its own temporary project directory.
/// A process-local mutex does not isolate nextest, which runs each test in a
/// separate process. In particular, simultaneous `vx cargo` commands must never
/// install into the same `store/rust/1.0.0` directory on Windows.
fn command_without_toolchain_env(program: impl AsRef<std::ffi::OsStr>, dir: &Path) -> Command {
    let home = dir.join("home");
    let cargo_home = home.join(".cargo");
    std::fs::create_dir_all(&cargo_home).unwrap();

    // Resolve the host's rustup home before changing HOME/USERPROFILE. The oracle
    // and vx's mismatch probe only read the installed default toolchain there.
    // Otherwise isolation would hide rustc and make the mismatch tests vacuous.
    // Any vx-managed installation gets its own rustup/cargo homes in VX_HOME.
    let rustup_home = std::env::var_os("RUSTUP_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::home_dir()
                .expect("host home must be available")
                .join(".rustup")
        });
    let rustup_home = std::path::absolute(rustup_home).unwrap();

    let mut cmd = Command::new(program);
    cmd.current_dir(dir)
        .env("VX_HOME", dir.join("vx-home"))
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("CARGO_HOME", cargo_home)
        .env("RUSTUP_HOME", rustup_home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_CACHE_HOME", home.join(".cache"))
        .env("XDG_DATA_HOME", home.join(".local/share"))
        .env("APPDATA", home.join("AppData/Roaming"))
        .env("LOCALAPPDATA", home.join("AppData/Local"))
        // cargo test's rustup shim exports this variable. Leaving it set makes
        // rustup own the toolchain and masks the numeric-pin scenario.
        .env_remove("RUSTUP_TOOLCHAIN");
    cmd
}

/// Run either `vx check` or the `vx cargo` run path in the same isolated fixture.
fn run_without_toolchain_env(dir: &Path, args: &[&str]) -> Output {
    let mut cmd = command_without_toolchain_env(common::vx_binary(), dir);
    cmd.args(args);
    run_command_with_timeout(cmd, COMMAND_TIMEOUT)
        .expect("vx should run within the fixture timeout")
}

/// Seed a private store with the real installed Cargo executable, not its rustup
/// proxy. These tests exercise pin diagnostics and command delegation, so they
/// must not download a full Rust toolchain for each severity case.
fn run_cargo_without_toolchain_env(dir: &Path, pin: &str) -> Output {
    let mut which_cargo = command_without_toolchain_env("rustup", dir);
    which_cargo.args(["which", "cargo"]);
    let cargo = run_command_with_timeout(which_cargo, COMMAND_TIMEOUT)
        .expect("resolve real Cargo within the fixture timeout");
    assert_success(&cargo, "locate the real Cargo executable");
    let cargo_source = PathBuf::from(stdout_str(&cargo).trim());
    assert!(cargo_source.is_file(), "Cargo must exist: {cargo_source:?}");
    let rustup_source = which::which("rustup").expect("rustup must be available");

    let store =
        vx_paths::VxPaths::with_base_dir(dir.join("vx-home")).version_store_dir("rust", pin);
    let bin = store.join("bin");
    let cargo_bin = store.join("cargo/bin");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir_all(&cargo_bin).unwrap();
    let suffix = std::env::consts::EXE_SUFFIX;
    // The provider verifies both its bootstrapper layout and final executable.
    // Copies (not hard links) prevent fixture writes from touching host binaries.
    std::fs::copy(&rustup_source, bin.join(format!("rustup-init{suffix}"))).unwrap();
    std::fs::copy(&rustup_source, cargo_bin.join(format!("rustup{suffix}"))).unwrap();
    std::fs::copy(cargo_source, cargo_bin.join(format!("cargo{suffix}"))).unwrap();

    let mut cmd = command_without_toolchain_env(common::vx_binary(), dir);
    cmd.args(["cargo", "--version"])
        // An incomplete fixture must fail rather than silently hitting the network.
        .env("VX_NO_AUTO_INSTALL", "1");
    run_command_with_timeout(cmd, COMMAND_TIMEOUT)
        .expect("vx cargo should run within the fixture timeout")
}

/// Write a `vx.toml` declaring the given rust pin, with no `rust-toolchain` file.
fn project_with_pin(dir: &Path, rust_pin: &str, extra: &str) {
    let config = format!("[tools]\nrust = \"{rust_pin}\"\n{extra}");
    std::fs::write(dir.join("vx.toml"), config).unwrap();

    // Belt and braces: the reproduction depends on there being no toolchain file.
    let _ = std::fs::remove_file(dir.join("rust-toolchain.toml"));
    let _ = std::fs::remove_file(dir.join("rust-toolchain"));
}

/// The version the toolchain under test actually reports, resolved the way the
/// code under test resolves it.
///
/// Both halves matter. `cargo test` runs under rustup's shim, so a plain `rustc`
/// inherits the suite's cwd — which sits under a repository root carrying a
/// `rust-toolchain.toml`, so rustup resolves *that* channel. The code under test
/// resolves from `project_root` with `RUSTUP_TOOLCHAIN` removed instead. Ask the
/// question differently and the oracle reports a version the subject never sees,
/// which makes the assertions below fail deterministically.
fn actual_rustc_version(dir: &Path) -> String {
    let mut cmd = command_without_toolchain_env("rustc", dir);
    cmd.arg("--version");
    let output = run_command_with_timeout(cmd, COMMAND_TIMEOUT)
        .expect("rustc must run within the fixture timeout");
    assert_success(
        &output,
        "resolve the real Rust toolchain for the pin fixture",
    );
    stdout_str(&output)
        .split_whitespace()
        .nth(1)
        .expect("rustc --version must report a version")
        .to_string()
}

#[rstest]
fn test_pin_commands_have_independent_writable_homes() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let first_cmd = command_without_toolchain_env(common::vx_binary(), first.path());
    let second_cmd = command_without_toolchain_env(common::vx_binary(), second.path());
    let first_env: std::collections::HashMap<_, _> = first_cmd.get_envs().collect();
    let second_env: std::collections::HashMap<_, _> = second_cmd.get_envs().collect();

    for key in [
        "VX_HOME",
        "HOME",
        "USERPROFILE",
        "CARGO_HOME",
        "XDG_CONFIG_HOME",
        "XDG_CACHE_HOME",
        "XDG_DATA_HOME",
        "APPDATA",
        "LOCALAPPDATA",
    ] {
        let first_path = Path::new(first_env[std::ffi::OsStr::new(key)].unwrap());
        let second_path = Path::new(second_env[std::ffi::OsStr::new(key)].unwrap());
        assert!(
            first_path.starts_with(first.path()),
            "{key}: {first_path:?}"
        );
        assert!(
            second_path.starts_with(second.path()),
            "{key}: {second_path:?}"
        );
        assert_ne!(first_path, second_path, "{key} must be fixture-local");
    }
    assert_eq!(first_env[std::ffi::OsStr::new("RUSTUP_TOOLCHAIN")], None);
    assert_eq!(second_env[std::ffi::OsStr::new("RUSTUP_TOOLCHAIN")], None);
    assert_eq!(
        first_env[std::ffi::OsStr::new("RUSTUP_HOME")],
        second_env[std::ffi::OsStr::new("RUSTUP_HOME")]
    );
}

/// The regression: a pin that does not match produces a visible stderr signal.
#[test]
fn test_mismatched_pin_warns() {
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(dir.path(), "1.0.0", "");

    let actual = actual_rustc_version(dir.path());

    let output = run_without_toolchain_env(dir.path(), &["check"]);
    let stderr = stderr_str(&output);
    let stdout = stdout_str(&output);

    // The signal has to name both the declared and the actual value.
    assert!(
        stderr.contains("1.0.0") || stdout.contains("1.0.0"),
        "declared pin must be visible. stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains(&actual) || stdout.contains(&actual),
        "actual version {actual} must be visible. stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

/// Default severity is warn, so a drifting repository is not broken by upgrading.
#[test]
fn test_mismatched_pin_exits_zero_by_default() {
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(dir.path(), "1.0.0", "");

    actual_rustc_version(dir.path());

    let output = run_without_toolchain_env(dir.path(), &["check"]);

    assert_eq!(
        output.status.code(),
        Some(0),
        "warn is the default severity, so a mismatch must not fail the command. stderr:\n{}",
        stderr_str(&output)
    );
}

/// Opting in to `"error"` is what makes the pin enforced.
#[test]
fn test_mismatched_pin_exits_non_zero_when_configured_as_error() {
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(
        dir.path(),
        "1.0.0",
        "\n[check]\ntoolchain_pin_mismatch = \"error\"\n",
    );

    actual_rustc_version(dir.path());

    let output = run_without_toolchain_env(dir.path(), &["check"]);

    assert_ne!(
        output.status.code(),
        Some(0),
        "with toolchain_pin_mismatch = \"error\" a mismatch must fail. stdout:\n{}\nstderr:\n{}",
        stdout_str(&output),
        stderr_str(&output)
    );
}

/// The zero-noise requirement: a correctly pinned repository sees no mismatch message.
#[test]
fn test_matching_pin_is_silent() {
    let dir = tempfile::tempdir().unwrap();

    let actual = actual_rustc_version(dir.path());

    project_with_pin(dir.path(), &actual, "");

    let output = run_without_toolchain_env(dir.path(), &["check"]);
    let combined = format!("{}\n{}", stdout_str(&output), stderr_str(&output));

    assert!(
        !combined.contains("the vx.toml pin is ignored"),
        "a pin equal to the actual version must not warn. output:\n{combined}"
    );
}

/// `rustup-managed` is an ownership declaration, not a version — it must never be
/// reported as a mismatch.
#[test]
fn test_rustup_managed_pin_is_never_a_mismatch() {
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(dir.path(), "rustup-managed", "");

    let output = run_without_toolchain_env(dir.path(), &["check"]);
    let combined = format!("{}\n{}", stdout_str(&output), stderr_str(&output));

    assert!(
        !combined.contains("the vx.toml pin is ignored"),
        "rustup-managed is not a version and must not mismatch. output:\n{combined}"
    );
}

/// `"ignore"` must actually silence the mismatch — the level is documented as such,
/// and without this case the level can be silently unwired on the CLI side.
#[test]
fn test_mismatched_pin_is_silent_when_configured_as_ignore() {
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(
        dir.path(),
        "1.0.0",
        "\n[check]\ntoolchain_pin_mismatch = \"ignore\"\n",
    );

    actual_rustc_version(dir.path());

    let output = run_without_toolchain_env(dir.path(), &["check"]);
    let combined = format!("{}\n{}", stdout_str(&output), stderr_str(&output));

    assert!(
        !combined.contains("the vx.toml pin is ignored"),
        "toolchain_pin_mismatch = \"ignore\" must suppress the mismatch. output:\n{combined}"
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "ignore must not fail the command"
    );
}

/// `vx check` must report the declared and actual values as two distinct fields.
#[test]
fn test_check_json_reports_declared_and_actual() {
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(dir.path(), "1.0.0", "");

    let actual = actual_rustc_version(dir.path());

    let output = run_without_toolchain_env(dir.path(), &["check", "--json"]);
    let stdout = stdout_str(&output);

    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!("vx check --json must emit JSON. error: {e}\nstdout:\n{stdout}")
    });

    let requirements = parsed["requirements"]
        .as_array()
        .expect("requirements array")
        .clone();

    let rust = requirements
        .iter()
        .find(|r| r["runtime"] == "rust")
        .unwrap_or_else(|| panic!("rust requirement missing from {requirements:?}"));

    // Two distinct fields: what was declared, and what will actually run.
    assert_eq!(rust["required"], "1.0.0");
    assert_eq!(
        rust["installed"].as_str().unwrap_or_default(),
        actual,
        "installed must report the actual toolchain version"
    );
}

// =============================================================================
// Installed-version reporting
// =============================================================================
//
// The suites above use whatever `rustc` the machine exposes, so they cannot pin
// the exact value `installed` reports. These assert the invariant that matters:
// `installed` must be the toolchain that will actually run, never the declared
// pin. The oracle asks `rustc` the same way the code under test does, so the
// assertion holds on any machine regardless of which version happens to be
// installed.

/// Run `vx check --json` against an isolated `VX_HOME` base directory.
///
/// `VxPaths` puts the store at `<VX_HOME>/store`, so the base is passed unmodified
/// and the caller creates `<base>/store/...`. An isolated store makes the machine's
/// real `~/.vx` irrelevant, so the test controls which tools appear installed
/// instead of depending on whatever the developer or CI runner happens to have.
fn check_json_in_home(home: &Path, project: &Path) -> Output {
    let mut cmd = Command::new(common::vx_binary());
    cmd.args(["check", "--json"])
        .current_dir(project)
        .env_remove("RUSTUP_TOOLCHAIN")
        .env("VX_HOME", home)
        .stdin(std::process::Stdio::null());

    cmd.output().expect("vx check should run")
}

/// The `rust` requirement from `vx check --json`, run the way production runs it.
fn rust_requirement(output: &Output) -> serde_json::Value {
    let stdout = stdout_str(output);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!(
            "vx check --json must emit JSON. error: {e}
stdout:
{stdout}"
        )
    });

    parsed["requirements"]
        .as_array()
        .expect("requirements array")
        .iter()
        .find(|r| r["runtime"] == "rust")
        .unwrap_or_else(|| panic!("rust requirement missing from {parsed}"))
        .clone()
}

/// `installed` reports the toolchain that will actually run, not the declared pin.
///
/// The pinned version's store directory exists but holds no toolchain, so `check`
/// reports `Installed` with the pin as its detected version while the real
/// toolchain is something else. That is the state in which the branch fell through
/// to the configured version and reported the declared pin as `installed`: a
/// repository pinned to `1.0.0` while `1.99.0` ran showed `installed = 1.0.0` and
/// the drift was invisible.
///
/// The store is an independent `VX_HOME` base, so no machine's real `~/.vx`, no
/// fake executable and no toolchain installation are involved.
#[test]
fn installed_reports_actual_toolchain_not_declared_pin() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();

    // Read the real toolchain first and fail loudly if it is unavailable: an
    // oracle that cannot run must not be mistaken for a passing assertion.
    let actual = actual_rustc_version(dir.path());

    let pin = unequal_numeric_pin(&actual);
    project_with_pin(dir.path(), &pin, "");

    // `VxPaths` stores at <VX_HOME>/store, so the base is passed unmodified.
    let home = dir.path().join("home");
    let store_dir = home.join("store").join("rust").join(&pin);
    std::fs::create_dir_all(store_dir.join("bin")).unwrap();

    let output = check_json_in_home(&home, dir.path());

    assert_eq!(
        output.status.code(),
        Some(0),
        "a drifting pin warns by default, so check must still succeed. stderr:
{}",
        stderr_str(&output)
    );

    let rust = rust_requirement(&output);
    assert_eq!(rust["runtime"], "rust");
    assert_eq!(rust["status"], "installed");
    assert_eq!(rust["required"], pin);

    // The installed path is the isolated store directory, not the user's.
    assert_eq!(
        rust["path"].as_str().unwrap_or_default(),
        store_dir.join("bin").display().to_string(),
        "path must be the isolated store bin directory. stdout:
{}",
        stdout_str(&output)
    );

    assert_ne!(
        pin, actual,
        "the chosen pin must differ from the actual toolchain"
    );
    assert_eq!(
        rust["installed"].as_str().unwrap_or_default(),
        actual,
        "installed must report the actual toolchain version, not the declared pin. stdout:
{}",
        stdout_str(&output)
    );
}

/// Pick a valid numeric pin that differs from `actual`.
///
/// Numeric keeps the pin comparable to a toolchain version, and unequal keeps the
/// drift visible; both are required for the assertion above to mean anything.
fn unequal_numeric_pin(actual: &str) -> String {
    if actual == "1.0.0" {
        return "1.0.1".to_string();
    }
    "1.0.0".to_string()
}

// =============================================================================
// Deterministic unit coverage for the installed-version choice
// =============================================================================
//
// The checks above drive the real `vx check` binary, so they can only observe the
// defect when the machine's store happens to hide the real toolchain. These pin
// the rule itself: for Rust the resolved toolchain wins, for everything else the
// previous behaviour is unchanged.

/// Rust reports the resolved toolchain even when detection produced a value.
#[test]
fn installed_version_prefers_resolved_rust_toolchain() {
    assert_eq!(
        vx_cli::commands::check::installed_version_for(
            "rust",
            Some("1.0.0".to_string()),
            Some("1.99.0")
        ),
        Some("1.99.0".to_string())
    );
    assert_eq!(
        vx_cli::commands::check::installed_version_for(
            "node",
            Some("1.0.0".to_string()),
            Some("1.99.0")
        ),
        Some("1.0.0".to_string())
    );
}

/// Rust still reports the resolved toolchain when detection found nothing —
/// the state that used to fall back to the declared pin.
#[test]
fn installed_version_prefers_resolved_rust_toolchain_without_detection() {
    assert_eq!(
        vx_cli::commands::check::installed_version_for("rust", None, Some("1.99.0")),
        Some("1.99.0".to_string())
    );
}

/// Without a resolved toolchain, Rust keeps the detected or configured value.
#[test]
fn installed_version_falls_back_to_detection_without_toolchain() {
    assert_eq!(
        vx_cli::commands::check::installed_version_for("rust", Some("1.0.0".to_string()), None),
        Some("1.0.0".to_string())
    );
}

/// Non-Rust tools are untouched: the resolved toolchain never overrides them.
#[test]
fn installed_version_leaves_non_rust_tools_alone() {
    assert_eq!(
        vx_cli::commands::check::installed_version_for(
            "node",
            Some("20.0.0".to_string()),
            Some("1.99.0")
        ),
        Some("20.0.0".to_string())
    );
    assert_eq!(
        vx_cli::commands::check::installed_version_for("node", None, Some("1.99.0")),
        None
    );
}

/// The same holds on the system-fallback path, where detection finds nothing.
#[test]
fn installed_reports_actual_toolchain_on_system_fallback() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(dir.path(), "rustup-managed", "");

    let actual = actual_rustc_version(dir.path());

    let output = run_without_toolchain_env(dir.path(), &["check", "--json"]);
    let rust = rust_requirement(&output);

    assert_eq!(rust["required"], "rustup-managed");
    assert_eq!(
        rust["installed"].as_str().unwrap_or_default(),
        actual,
        "installed must report the actual toolchain version. stdout:
{}",
        stdout_str(&output)
    );
}

// =============================================================================
// Run path: the same severity must govern `vx cargo …`
// =============================================================================
//
// The mismatch is reported from the resolver on this path, not from `vx check`, and
// the two used to disagree: `ignore` silenced `vx check` and nothing else. Every
// assertion below is paired with its opposite so neither can pass because the
// warning simply never fired.

/// The positive control: with no `[check]` section the run path must warn.
///
/// Without this, the `ignore` case below would also pass if the warning were broken
/// outright, so the pair is what actually pins the behaviour.
#[test]
fn test_run_path_warns_by_default() {
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(dir.path(), "1.0.0", "");

    actual_rustc_version(dir.path());

    let output = run_cargo_without_toolchain_env(dir.path(), "1.0.0");
    assert_success(&output, "run cargo with the configured pin severity");
    let combined = format!("{}\n{}", stdout_str(&output), stderr_str(&output));

    assert!(
        combined.contains("the vx.toml pin is ignored"),
        "the run path must warn about a drifting pin. output:\n{combined}"
    );
}

/// The regression: `ignore` must silence the run path, not just `vx check`.
#[test]
fn test_run_path_is_silent_when_configured_as_ignore() {
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(
        dir.path(),
        "1.0.0",
        "\n[check]\ntoolchain_pin_mismatch = \"ignore\"\n",
    );

    actual_rustc_version(dir.path());

    let output = run_cargo_without_toolchain_env(dir.path(), "1.0.0");
    assert_success(&output, "run cargo with the configured pin severity");
    let combined = format!("{}\n{}", stdout_str(&output), stderr_str(&output));

    assert!(
        !combined.contains("the vx.toml pin is ignored"),
        "ignore must silence the run path too. output:\n{combined}"
    );
}

/// An explicit `warn` behaves like the default.
#[test]
fn test_run_path_warns_when_configured_as_warn() {
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(
        dir.path(),
        "1.0.0",
        "\n[check]\ntoolchain_pin_mismatch = \"warn\"\n",
    );

    actual_rustc_version(dir.path());

    let output = run_cargo_without_toolchain_env(dir.path(), "1.0.0");
    assert_success(&output, "run cargo with the configured pin severity");
    let combined = format!("{}\n{}", stdout_str(&output), stderr_str(&output));

    assert!(
        combined.contains("the vx.toml pin is ignored"),
        "warn must still report the mismatch on the run path. output:\n{combined}"
    );
}

/// `error` escalates only `vx check`'s exit code. Failing here would make an
/// already-drifting repository unbuildable, so the run path keeps warning.
#[test]
fn test_run_path_still_warns_when_configured_as_error() {
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(
        dir.path(),
        "1.0.0",
        "\n[check]\ntoolchain_pin_mismatch = \"error\"\n",
    );

    actual_rustc_version(dir.path());

    let output = run_cargo_without_toolchain_env(dir.path(), "1.0.0");
    assert_success(&output, "run cargo with the configured pin severity");
    let combined = format!("{}\n{}", stdout_str(&output), stderr_str(&output));

    assert!(
        combined.contains("the vx.toml pin is ignored"),
        "error must not suppress the run-path warning. output:\n{combined}"
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "error must not fail the build on the run path. output:\n{combined}"
    );
}

/// `ignore` must not change what the command actually does — only its noise.
#[test]
fn test_run_path_ignore_does_not_change_command_output() {
    let dir = tempfile::tempdir().unwrap();
    project_with_pin(
        dir.path(),
        "1.0.0",
        "\n[check]\ntoolchain_pin_mismatch = \"ignore\"\n",
    );

    actual_rustc_version(dir.path());

    let output = run_cargo_without_toolchain_env(dir.path(), "1.0.0");
    assert_success(&output, "run cargo with the configured pin severity");

    assert!(
        stdout_str(&output).contains("cargo"),
        "the delegated command must still run. stderr:\n{}",
        stderr_str(&output)
    );
}

/// The zero-noise requirement on the run path: a matching pin stays silent.
#[test]
fn test_run_path_matching_pin_is_silent() {
    let dir = tempfile::tempdir().unwrap();

    let actual = actual_rustc_version(dir.path());

    project_with_pin(dir.path(), &actual, "");

    let output = run_cargo_without_toolchain_env(dir.path(), &actual);
    assert_success(&output, "run cargo with the configured pin severity");
    let combined = format!("{}\n{}", stdout_str(&output), stderr_str(&output));

    assert!(
        !combined.contains("the vx.toml pin is ignored"),
        "a pin equal to the actual version must not warn. output:\n{combined}"
    );
}
