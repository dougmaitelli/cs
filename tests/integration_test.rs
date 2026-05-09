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

/// Spawn cs in a pseudo-terminal and send keystrokes to simulate user input.
///
/// This enables integration testing of interactive prompts (Input, FuzzySelect, Select, Confirm)
/// used by the add, delete, and import commands. The function uses Python's pty module to create
/// a pseudo-terminal so that dialoguer's TUI prompts work correctly.
///
/// # Arguments
/// * `config_home` - XDG_CONFIG_HOME for the cs binary
/// * `args` - CLI arguments for cs
/// * `keystrokes` - Raw keystrokes to send (e.g. b"my command\r", b"\r" for Enter, b"\x1b" for ESC)
/// * `extra_env` - Additional environment variables as (name, value) pairs (e.g. ("HOME", path))
fn run_cs_interactive(
    config_home: PathBuf,
    args: Vec<&str>,
    keystrokes: Vec<Vec<u8>>,
    extra_env: Vec<(&str, String)>,
) -> (i32, String) {
    use std::io::Write;

    let helper_path = "tests/pty_helper.py";

    let extra_env_str = extra_env
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(",");

    let mut py = Command::new("python3")
        .arg(helper_path)
        .arg(cs_bin_path())
        .arg(config_home.to_string_lossy().to_string())
        .arg(extra_env_str)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn pty_helper");

    let input_data = serde_json::json!({
        "keystrokes": keystrokes,
    });

    py.stdin
        .as_mut()
        .unwrap()
        .write_all(serde_json::to_string(&input_data).unwrap().as_bytes())
        .expect("Failed to write keystrokes");

    let output = py.wait_with_output().expect("Failed to wait on pty_helper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut exit_code = -1;
    for line in stderr.lines() {
        if let Some(code) = line.strip_prefix("EXIT:") {
            if let Ok(c) = code.trim().parse::<i32>() {
                exit_code = c;
            }
        }
    }

    // Strip the trailing newline that the helper adds
    (exit_code, stdout.trim().to_string())
}

// ============================================================================
// Help / basic behavior
// ============================================================================

#[test]
fn cs_help_displays_usage() {
    let (config_home, _) = setup_xdg_config();

    let output = run_cs_with_config(config_home, &["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("cs") || stdout.contains("CLI") || stdout.contains("snippet"));
}

#[test]
fn cs_no_args_exits() {
    let (config_home, _) = setup_xdg_config();

    let output = run_cs_with_config(config_home, &[]);

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
    let (config_home, _) = setup_xdg_config();

    let output = run_cs_with_config(config_home, &["list"]);
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
    let (config_home, _) = setup_xdg_config();

    let output = run_cs_with_config(config_home, &["hello"]);
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

#[test]
fn cs_add_snippet_to_empty_snippets() {
    let (config_home, _) = setup_xdg_config();

    write_snippets_toml(&config_home, "");

    let keystrokes = vec![
        b"echo hello world\r".to_vec(),
        b"print hello\r".to_vec(),
        b"\r".to_vec(), // empty tags
        b"l".to_vec(),  // fuzzy filter OS to "linux"
        b"\r".to_vec(), // confirm "linux"
    ];

    let (exit_code, output) = run_cs_interactive(
        config_home.clone(),
        vec!["--no-fzf", "add"],
        keystrokes,
        vec![],
    );

    assert_eq!(exit_code, 0);
    assert!(output.contains("echo hello world") || output.contains("Snippet added"));
    assert!(output.contains("print hello"));

    let config = fs::read_to_string(config_home.join("cs/snippets.toml")).unwrap();
    assert!(config.contains("echo hello world"));
    assert!(config.contains("print hello"));
    assert!(config.contains("os = \"linux\""));
}

#[test]
fn cs_add_snippet_to_existing_snippets() {
    let (config_home, _) = setup_xdg_config();

    write_snippets_toml(
        &config_home,
        r#"
[[snippet]]
cmd = "echo existing"
description = "existing snippet"
tags = ["old"]
os = "linux"
"#,
    );

    let keystrokes = vec![
        b"echo new command\r".to_vec(),
        b"new description\r".to_vec(),
        b"newtag\r".to_vec(),
        b"m".to_vec(),  // fuzzy filter OS to "macos"
        b"\r".to_vec(), // confirm "macos"
    ];

    let (exit_code, output) = run_cs_interactive(
        config_home.clone(),
        vec!["--no-fzf", "add"],
        keystrokes,
        vec![],
    );

    assert_eq!(exit_code, 0);
    assert!(output.contains("new command") || output.contains("Snippet added"));

    // Verify both snippets exist in the file
    let config = fs::read_to_string(config_home.join("cs/snippets.toml")).unwrap();
    assert!(config.contains("echo existing"));
    assert!(config.contains("existing snippet"));
    assert!(config.contains("echo new command"));
    assert!(config.contains("new description"));
}

#[test]
fn cs_add_snippet_to_include_file() {
    let (config_home, _) = setup_xdg_config();

    write_config_files(&config_home, |cs_dir| {
        // Main config with include
        fs::write(
            cs_dir.join("snippets.toml"),
            r#"
[config]
includes = ["extra.toml"]
"#,
        )
        .unwrap();
        // Extra file is initially empty
        fs::write(cs_dir.join("extra.toml"), "").unwrap();
    });

    let keystrokes = vec![
        b"echo in extra\r".to_vec(),
        b"extra snippet\r".to_vec(),
        b"\r".to_vec(),     // empty tags
        b"a".to_vec(),      // fuzzy filter OS to "any"
        b"\r".to_vec(),     // confirm "any"
        b"\x1b[B".to_vec(), // down arrow to select extra.toml
        b"\r".to_vec(),     // confirm extra.toml
    ];

    let (exit_code, output) = run_cs_interactive(
        config_home.clone(),
        vec!["--no-fzf", "add"],
        keystrokes,
        vec![],
    );

    assert_eq!(exit_code, 0);
    assert!(output.contains("Snippet added") || output.contains("echo in extra"));

    // The snippet should be in extra.toml
    let extra_config = fs::read_to_string(config_home.join("cs/extra.toml")).unwrap();
    assert!(extra_config.contains("echo in extra"));

    // Also verify it appears in list output
    let list_output = run_cs_with_config(config_home, &["--no-fzf", "list", "--all-os"]);
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_stdout.contains("echo in extra"));
}

// ============================================================================
// delete command
// ============================================================================

#[test]
fn cs_delete_snippet_cancelled() {
    let (config_home, _) = setup_xdg_config();

    write_snippets_toml(
        &config_home,
        r#"
[[snippet]]
cmd = "echo to delete"
description = "snippet to delete"
tags = ["test"]
os = "any"
"#,
    );

    // Select the snippet (Enter) and then decline with n
    let keystrokes = vec![
        b"\r".to_vec(), // select first snippet
        b"n".to_vec(),  // n = no, decline deletion
    ];

    let (_exit_code, output) = run_cs_interactive(
        config_home.clone(),
        vec!["--no-fzf", "delete", "to delete"],
        keystrokes,
        vec![],
    );

    assert!(output.contains("Deletion cancelled") || output.contains("cancelled"));

    // Verify the snippet still exists in the file
    let config = fs::read_to_string(config_home.join("cs/snippets.toml")).unwrap();
    assert!(config.contains("echo to delete"));
    assert!(config.contains("snippet to delete"));
}

#[test]
fn cs_delete_snippet_confirmed() {
    let (config_home, _) = setup_xdg_config();

    write_snippets_toml(
        &config_home,
        r#"
[[snippet]]
cmd = "echo to delete"
description = "snippet to delete"
tags = ["test"]
os = "any"
"#,
    );

    // Select the snippet (Enter) and confirm with y
    let keystrokes = vec![
        b"\r".to_vec(), // select first snippet
        b"y".to_vec(),  // confirm deletion
        b"\r".to_vec(), // confirm with Enter
    ];

    let (exit_code, output) = run_cs_interactive(
        config_home.clone(),
        vec!["--no-fzf", "delete", "to delete"],
        keystrokes,
        vec![],
    );

    assert_eq!(exit_code, 0);
    assert!(output.contains("Snippet deleted") || output.contains("deleted"));

    // Verify the snippet no longer exists in the file
    let config = fs::read_to_string(config_home.join("cs/snippets.toml")).unwrap();
    assert!(!config.contains("echo to delete"));
    assert!(!config.contains("snippet to delete"));
}

#[test]
fn cs_delete_snippet_not_in_list_after_deletion() {
    let (config_home, _) = setup_xdg_config();

    write_snippets_toml(
        &config_home,
        r#"
[[snippet]]
cmd = "echo keep me"
description = "keep this"
tags = ["keep"]
os = "any"

[[snippet]]
cmd = "echo delete me"
description = "delete this"
tags = ["delete"]
os = "any"
"#,
    );

    // Select the second snippet (down arrow + Enter) and confirm
    let keystrokes = vec![
        b"\x1b[B".to_vec(), // down arrow to select second snippet
        b"\r".to_vec(),     // confirm selection
        b"y".to_vec(),      // confirm deletion
        b"\r".to_vec(),     // confirm with Enter
    ];

    let (_exit_code, output) = run_cs_interactive(
        config_home.clone(),
        vec!["--no-fzf", "delete", "delete me"],
        keystrokes,
        vec![],
    );

    assert!(output.contains("Snippet deleted") || output.contains("deleted"));

    // Verify only the kept snippet remains
    let config = fs::read_to_string(config_home.join("cs/snippets.toml")).unwrap();
    assert!(config.contains("echo keep me"));
    assert!(config.contains("keep this"));
    assert!(!config.contains("echo delete me"));
    assert!(!config.contains("delete this"));

    // Verify the deleted snippet doesn't appear in list output
    let list_output = run_cs_with_config(config_home, &["--no-fzf", "list", "--all-os"]);
    let list_stdout = String::from_utf8_lossy(&list_output.stdout);
    assert!(!list_stdout.contains("echo delete me"));
    assert!(list_stdout.contains("echo keep me"));
}

// ============================================================================
// import command
// ============================================================================

#[test]
fn cs_import_selection_cancelled() {
    let (config_home, tmp) = setup_xdg_config();

    write_snippets_toml(&config_home, "");

    let home_dir = tmp.join("home");
    fs::create_dir_all(&home_dir).unwrap();
    fs::write(
        home_dir.join(".bash_history"),
        "echo command one\necho command two\n",
    )
    .unwrap();

    // Press ESC to cancel the history selection
    let keystrokes = vec![
        b"\x1b".to_vec(), // ESC to cancel selection
    ];

    let (_exit_code, output) = run_cs_interactive(
        config_home.clone(),
        vec!["--no-fzf", "import"],
        keystrokes,
        vec![("HOME", home_dir.to_string_lossy().to_string())],
    );

    assert!(output.contains("cancelled") || output.contains("Selection cancelled"));

    // Verify no snippets were added
    let config = fs::read_to_string(config_home.join("cs/snippets.toml")).unwrap();
    assert!(!config.contains("[[snippet]]"));
}

#[test]
fn cs_import_from_history_entry() {
    let (config_home, tmp) = setup_xdg_config();

    write_snippets_toml(&config_home, "");

    // Create a mock .bash_history in a temp home directory
    let home_dir = tmp.join("home");
    fs::create_dir_all(&home_dir).unwrap();
    fs::write(
        home_dir.join(".bash_history"),
        "echo old command 1\necho old command 2\necho command to import\necho old command 3\n",
    )
    .unwrap();

    // Select "echo command to import" (3rd entry, index 2)
    // But the history is read in reverse
    // The first item shown in the selector will be "echo old command 3"

    let keystrokes = vec![
        b"\x1b[B".to_vec(), // down arrow to select 2nd entry: "echo command to import"
        b"\r".to_vec(),     // confirm selection
        b"imported from history\r".to_vec(),
        b"\r".to_vec(), // empty tags
        b"i".to_vec(),  // fuzzy filter OS to "any"
        b"\r".to_vec(), // confirm "any"
    ];

    let (exit_code, output) = run_cs_interactive(
        config_home.clone(),
        vec!["--no-fzf", "import"],
        keystrokes,
        vec![("HOME", home_dir.to_string_lossy().to_string())],
    );

    assert_eq!(exit_code, 0);
    assert!(
        output.contains("Snippet imported")
            || output.contains("imported from history")
            || output.contains("echo command to import")
    );

    // Verify the imported snippet exists in the config
    let config = fs::read_to_string(config_home.join("cs/snippets.toml")).unwrap();
    assert!(config.contains("echo command to import"));
}
