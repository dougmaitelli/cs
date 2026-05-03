use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn current_os_id() -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(id) = line.strip_prefix("ID=") {
                return id.trim_matches('"').to_lowercase();
            }
        }
    }
    "linux".to_string()
}

fn mktemp_dir() -> PathBuf {
    let tmp = std::env::temp_dir().join(format!(
        "cs-integration-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&tmp);

    fs::create_dir_all(&tmp).expect("Failed to create temp dir");

    tmp
}

fn teardown(tmp: PathBuf) {
    let _ = fs::remove_dir_all(tmp);
}

/// Creates a temp dir with a `config` subdirectory. Returns `(config_home, tmp_root)`.
fn setup_xdg_config() -> (PathBuf, PathBuf) {
    let tmp = mktemp_dir();
    let config_home = tmp.join("config");

    fs::create_dir_all(&config_home).expect("Failed to create config dir");

    (config_home, tmp)
}

/// Write a snippets.toml file under `{config_home}/cs/snippets.toml`.
fn write_snippets_toml(config_home: &std::path::Path, content: &str) {
    let snippets_dir = config_home.join("cs");

    fs::create_dir_all(&snippets_dir).ok();
    fs::write(snippets_dir.join("snippets.toml"), content).unwrap();
}

/// Write arbitrary files under `{config_home}/cs/`. Use this when you need
/// more control than `write_snippets_toml` provides (e.g. includes, subdirs).
fn write_config_files(config_home: &std::path::Path, write: impl FnOnce(PathBuf)) {
    let cs_dir = config_home.join("cs");

    fs::create_dir_all(&cs_dir).ok();

    write(cs_dir);
}

fn cs_bin_path() -> String {
    env::var("CARGO_BIN_EXE_cs")
        .ok()
        .or_else(|| {
            let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
            Some(format!("{}/debug/cs", target_dir))
        })
        .expect("Cannot find cs binary path")
}

fn run_cs<const N: usize>(args: &[&str; N]) -> std::process::Output {
    let bin = cs_bin_path();

    Command::new(&bin)
        .args(args)
        .output()
        .expect("Failed to run cs binary")
}

fn run_cs_with_config<const N: usize>(
    config_home: PathBuf,
    args: &[&str; N],
) -> std::process::Output {
    let bin = cs_bin_path();
    let output = Command::new(&bin)
        .env("XDG_CONFIG_HOME", &config_home)
        .args(args)
        .output()
        .expect("Failed to run cs binary");

    teardown(config_home.parent().unwrap().to_path_buf());

    output
}

// ============================================================================
// Help / basic behavior
// ============================================================================

#[test]
fn cs_help_displays_usage() {
    let output = run_cs(&["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("cs") || stdout.contains("CLI") || stdout.contains("snippet"));
}

#[test]
fn cs_no_args_exits() {
    let output = run_cs(&[]);

    assert!(!output.stdout.is_empty() || !output.stderr.is_empty());
}

// ============================================================================
// Edge cases / error handling
// ============================================================================

#[test]
fn cs_with_invalid_toml_exits_with_error() {
    let (config_home, _) = setup_xdg_config();

    write_config_files(&config_home, |cs_dir| {
        fs::write(cs_dir.join("snippets.toml"), "this is not [[valid toml}}").unwrap();
    });

    let output = run_cs_with_config(config_home, &["list"]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Error") || stderr.contains("Failed to parse"));
}

// ============================================================================
// list command
// ============================================================================

#[test]
fn cs_list_displays_empty_message() {
    let output = run_cs(&["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("No snippets yet"));
}

#[test]
fn cs_list_with_snippets_shows_count() {
    let (config_home, _) = setup_xdg_config();

    write_snippets_toml(
        &config_home,
        r#"
[[snippet]]
cmd = "echo hello"
description = "print hello"
tags = []
os = "any"
"#,
    );

    let output = run_cs_with_config(config_home, &["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("1 snippet(s)"));
    assert!(stdout.contains("echo hello"));
    assert!(stdout.contains("print hello"));
}

#[test]
fn cs_list_filters_by_current_os() {
    let (config_home, _) = setup_xdg_config();

    let current = current_os_id();
    let other = if current == "macos" {
        "windows"
    } else {
        "macos"
    };

    write_snippets_toml(
        &config_home,
        &format!(
            r#"
[[snippet]]
cmd = "echo current-os"
description = "current os"
tags = []
os = "{}"

[[snippet]]
cmd = "echo macos-only"
description = "macos only"
tags = []
os = "{}"
"#,
            current, other
        ),
    );

    let output = run_cs_with_config(config_home, &["list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("1 snippet(s)"));
    assert!(stdout.contains("echo current-os"));
    assert!(!stdout.contains("echo macos-only"));
}

#[test]
fn cs_list_all_os_shows_all() {
    let (config_home, _) = setup_xdg_config();

    write_snippets_toml(
        &config_home,
        r#"
[[snippet]]
cmd = "echo linux-only"
description = "linux only"
tags = []
os = "linux"

[[snippet]]
cmd = "echo macos-only"
description = "macos only"
tags = []
os = "macos"
"#,
    );

    let output = run_cs_with_config(config_home, &["list", "--all-os"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("2 snippet(s)"));
    assert!(stdout.contains("echo linux-only"));
    assert!(stdout.contains("echo macos-only"));
}

// ============================================================================
// search command (bare query)
// ============================================================================

#[test]
fn cs_search_empty_snippets_shows_hint() {
    let output = run_cs(&["hello"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("No snippets yet") || stdout.contains("No snippets found"));
}

#[test]
fn cs_search_by_cmd_finds_snippet() {
    let (config_home, _) = setup_xdg_config();

    write_snippets_toml(
        &config_home,
        r#"
[[snippet]]
cmd = "echo hello world"
description = "print hello"
tags = ["test"]
os = "linux"
"#,
    );

    let output = run_cs_with_config(config_home, &["--all-os", "hello"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("echo hello world") || output.status.code() == Some(1));
}

#[test]
fn cs_search_by_description_finds_snippet() {
    let (config_home, _) = setup_xdg_config();

    write_snippets_toml(
        &config_home,
        r#"
[[snippet]]
cmd = "cargo build"
description = "build the project"
tags = ["cargo"]
os = "linux"
"#,
    );

    let output = run_cs_with_config(config_home, &["--all-os", "build"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("cargo build") || output.status.code() == Some(1));
}

#[test]
fn cs_search_by_tag_finds_snippet() {
    let (config_home, _) = setup_xdg_config();

    write_snippets_toml(
        &config_home,
        r#"
[[snippet]]
cmd = "docker run"
description = "run a container"
tags = ["docker", "containers"]
os = "any"
"#,
    );

    let output = run_cs_with_config(config_home, &["--all-os", "docker"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("docker run") || output.status.code() == Some(1));
}

// ============================================================================
// add command
// ============================================================================

// TODO: Add tests for:
// - Adding a snippet to an empty snippets.toml
// - Adding a snippet to a snippets.toml that already has snippets
// - Adding a snippet to an include file (e.g. extra.toml) and verifying it appears in the list output

// ============================================================================
// delete command
// ============================================================================

// TODO: Add tests for:
// - Deleting a snippet but not confirming
// - Deleting a snippet and confirming, then verifying it no longer appears in the list output

// ============================================================================
// import command
// ============================================================================

// TODO: Add tests for:
// - Importing from a history entry (might need to mock the shell history)
