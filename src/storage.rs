use crate::os::Os;
use crate::snippet::Snippet;
use crate::utils::icons::EMOJI_EMPTY;
use console::style;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;
use strum::IntoEnumIterator;

#[derive(serde::Deserialize, serde::Serialize, Default)]
struct SnippetFile {
    #[serde(default)]
    config: Config,
    #[serde(default)]
    snippet: Vec<Snippet>,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Clone)]
struct Config {
    #[serde(default)]
    nerd_fonts: bool,
    #[serde(default)]
    includes: Vec<String>,
}

#[derive(serde::Serialize)]
struct IncludeFile {
    snippet: Vec<Snippet>,
}

static STORE: OnceLock<SnippetFile> = OnceLock::new();

pub(crate) fn config_path() -> PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".config")
        })
        .join("cs")
        .join("snippets.toml")
}

fn config_dir_for_path(path: &std::path::Path) -> PathBuf {
    path.parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn ensure_config_dir() -> io::Result<()> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}

fn load_config_from_file(path: &PathBuf) -> Config {
    if !path.exists() {
        return Config::default();
    }

    let content = fs::read_to_string(path).unwrap_or_default();
    if content.trim().is_empty() {
        return Config::default();
    }

    toml::from_str::<SnippetFile>(&content)
        .map(|f| f.config)
        .unwrap_or_default()
}

fn load_snippets_from_file(path: &PathBuf) -> Result<Vec<Snippet>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    if content.trim().is_empty() {
        return Ok(Vec::new());
    }

    let file: SnippetFile = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;

    let mut snippets = file.snippet;
    for s in &mut snippets {
        s.source = path.clone();
    }

    Ok(snippets)
}

pub(crate) fn init_store(path: &PathBuf) -> Result<(), String> {
    let config = load_config_from_file(path);
    let base_dir = config_dir_for_path(path);
    let mut snippets = load_snippets_from_file(path)?;

    for include in &config.includes {
        let include_path = base_dir.join(include);
        let included = load_snippets_from_file(&include_path)?;
        snippets.extend(included);
    }

    sort_snippets(&mut snippets);

    STORE.get_or_init(|| SnippetFile {
        config,
        snippet: snippets,
    });
    Ok(())
}

pub(crate) fn use_nerd_fonts() -> bool {
    STORE.get().map(|s| s.config.nerd_fonts).unwrap_or_default()
}

pub(crate) fn load_snippets() -> &'static [Snippet] {
    STORE
        .get()
        .map(|s| s.snippet.as_slice())
        .unwrap_or_default()
}

pub(crate) fn add_snippet(snippet: Snippet, target: PathBuf) -> Result<(), String> {
    let snippets = load_snippets().to_vec();
    let mut new_snippet = snippet;

    new_snippet.source = target;
    let mut all_snippets = snippets;
    all_snippets.push(new_snippet);

    save_snippets(&all_snippets)
}

pub(crate) fn delete_snippet(snippets: &[Snippet], index: usize) -> Result<(), String> {
    let mut all_snippets = snippets.to_vec();
    all_snippets.remove(index);

    save_snippets(&all_snippets)
}

fn save_snippets(snippets: &[Snippet]) -> Result<(), String> {
    ensure_config_dir().map_err(|e| format!("Failed to create config directory: {}", e))?;

    // Group snippets by source file
    let mut by_file: HashMap<PathBuf, Vec<Snippet>> = HashMap::new();
    for snippet in snippets {
        by_file
            .entry(snippet.source.clone())
            .or_default()
            .push(snippet.clone());
    }

    // Save main config file (with config section)
    let main_path = config_path();
    let config = STORE.get().map(|s| s.config.clone()).unwrap_or_default();
    let main_snippets = by_file.remove(&main_path).unwrap_or_default();
    let main_file = SnippetFile {
        config,
        snippet: main_snippets,
    };

    let content = toml::to_string_pretty(&main_file)
        .map_err(|e| format!("Failed to serialize snippets: {}", e))?;
    fs::write(&main_path, content)
        .map_err(|e| format!("Failed to write {}: {}", main_path.display(), e))?;

    // Save included files (without config section)
    for (path, file_snippets) in by_file {
        let file = IncludeFile {
            snippet: file_snippets,
        };

        let content = toml::to_string_pretty(&file)
            .map_err(|e| format!("Failed to serialize snippets: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))?;
    }

    Ok(())
}

fn available_files() -> Vec<PathBuf> {
    let mut files = vec![config_path()];
    let store = STORE.get().unwrap();
    let base_dir = config_dir_for_path(&config_path());

    for include in &store.config.includes {
        files.push(base_dir.join(include));
    }

    files
}

pub(crate) fn select_target_file() -> Result<PathBuf, String> {
    let files = available_files();
    if files.len() == 1 {
        return Ok(files[0].clone());
    }

    let file_names: Vec<String> = files
        .iter()
        .map(|p| {
            p.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    let idx = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Save to file")
        .items(&file_names)
        .default(0)
        .interact()
        .map_err(|e| format!("Failed to select file: {}", e))?;

    Ok(files[idx].clone())
}

fn os_sort_key(os: &Os) -> usize {
    Os::iter().position(|o| o == *os).unwrap_or(usize::MAX)
}

fn sort_snippets(snippets: &mut [Snippet]) {
    snippets.sort_by(|a, b| {
        os_sort_key(&a.os)
            .cmp(&os_sort_key(&b.os))
            .then_with(|| a.cmd.to_lowercase().cmp(&b.cmd.to_lowercase()))
    });
}

pub(crate) fn filter_snippets(
    query: &str,
    all_os: bool,
) -> Result<Option<Vec<(usize, Snippet)>>, String> {
    let current_os = crate::utils::os_detect::detect_os();
    let snippets = load_snippets();

    if snippets.is_empty() {
        println!(
            "{}",
            style(format!(
                "{} No snippets yet. Use `cs add` to create one.",
                EMOJI_EMPTY
            ))
            .yellow()
        );
        return Ok(None);
    }

    let filtered: Vec<(usize, Snippet)> = snippets
        .iter()
        .enumerate()
        .filter(|(_, s)| {
            if all_os {
                query.is_empty() || s.matches(query)
            } else if query.is_empty() {
                s.matches_os(&current_os)
            } else {
                s.matches(query) && s.matches_os(&current_os)
            }
        })
        .map(|(i, s)| (i, (*s).clone()))
        .collect();

    if filtered.is_empty() {
        if query.is_empty() {
            println!(
                "{} No snippets found for {}. Use --all-os to see all.",
                crate::utils::icons::EMOJI_NOT_FOUND,
                current_os
            );
        } else {
            println!(
                "{} No snippets found matching '{}'",
                crate::utils::icons::EMOJI_NOT_FOUND,
                query
            );
        }
        return Ok(None);
    }

    Ok(Some(filtered))
}

// ----------------------------------------------------------------------------
// TESTS
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::os::Os;

    fn make_snippet(cmd: &str, description: &str, tags: Vec<&str>, os: Os) -> Snippet {
        Snippet {
            cmd: cmd.to_string(),
            description: description.to_string(),
            tags: tags.into_iter().map(String::from).collect(),
            os,
            source: PathBuf::new(),
        }
    }

    fn make_snippet_with_source(
        cmd: &str,
        description: &str,
        tags: Vec<&str>,
        os: Os,
        source: PathBuf,
    ) -> Snippet {
        Snippet {
            cmd: cmd.to_string(),
            description: description.to_string(),
            tags: tags.into_iter().map(String::from).collect(),
            os,
            source,
        }
    }

    #[test]
    fn sort_snippets_sorts_by_os_then_command() {
        let mut snippets = vec![
            make_snippet("ls", "list", vec![], Os::Macos),
            make_snippet("git", "version control", vec![], Os::Linux),
            make_snippet("curl", "fetch url", vec![], Os::Linux),
        ];

        sort_snippets(&mut snippets);

        assert_eq!(snippets[0].os, Os::Linux);
        assert_eq!(snippets[0].cmd, "curl");
        assert_eq!(snippets[1].cmd, "git");
        assert_eq!(snippets[2].os, Os::Macos);
    }

    #[test]
    fn os_sort_key_returns_positions() {
        assert_eq!(os_sort_key(&Os::Linux), 0);
        assert_eq!(os_sort_key(&Os::Macos), 1);
        assert_eq!(os_sort_key(&Os::Windows), 2);
        assert_eq!(os_sort_key(&Os::Fedora), 3);
        assert_eq!(os_sort_key(&Os::Ubuntu), 4);
        assert_eq!(os_sort_key(&Os::Debian), 5);
        assert_eq!(os_sort_key(&Os::Arch), 6);
        assert_eq!(os_sort_key(&Os::Centos), 7);
        assert_eq!(os_sort_key(&Os::Rhel), 8);
        assert_eq!(os_sort_key(&Os::Opensuse), 9);
        assert_eq!(os_sort_key(&Os::Mint), 10);
        assert_eq!(os_sort_key(&Os::Bsd), 11);
        assert_eq!(os_sort_key(&Os::Any), 12);
    }

    #[test]
    fn snippetfile_serializes_with_config_and_snippets() {
        let snippets = vec![make_snippet("ls", "list files", vec![], Os::Linux)];
        let file = SnippetFile {
            config: Config {
                nerd_fonts: true,
                includes: vec!["extra.toml".to_string()],
            },
            snippet: snippets,
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();

        assert!(toml_str.contains("nerd_fonts = true"));
        assert!(toml_str.contains("extra.toml"));
        assert!(toml_str.contains("ls"));
    }

    #[test]
    fn snippetfile_serializes_snippet_only() {
        let snippets = vec![make_snippet(
            "cargo build",
            "build project",
            vec![],
            Os::Linux,
        )];
        let file = SnippetFile {
            config: Config::default(),
            snippet: snippets,
        };
        let toml_str = toml::to_string_pretty(&file).unwrap();

        assert!(toml_str.contains("cargo build"));
    }

    #[test]
    fn config_serializes_nerd_fonts() {
        let config = Config {
            nerd_fonts: true,
            includes: vec!["extras.toml".to_string()],
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();

        assert!(toml_str.contains("nerd_fonts = true"));
        assert!(toml_str.contains("extras.toml"));
    }

    #[test]
    fn config_serializes_default() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();

        assert!(toml_str.contains("nerd_fonts = false"));
    }

    #[test]
    fn load_config_from_file_missing_returns_default() {
        let path = std::env::temp_dir().join("cs_test_config_missing");
        let config = load_config_from_file(&path);

        assert!(!config.nerd_fonts);
        assert!(config.includes.is_empty());
    }

    #[test]
    fn load_config_from_file_empty_returns_default() {
        let path = std::env::temp_dir().join("cs_test_config_empty");
        fs::write(&path, "").unwrap();

        let config = load_config_from_file(&path);

        assert!(!config.nerd_fonts);
        assert!(config.includes.is_empty());

        fs::remove_file(&path).ok();
    }

    #[test]
    fn load_config_from_file_parses_nerd_fonts() {
        let path = std::env::temp_dir().join("cs_test_config_nf.toml");
        let content = r#"
[config]
nerd_fonts = true
includes = ["extra.toml", "more.toml"]
"#;
        fs::write(&path, content).unwrap();

        let config = load_config_from_file(&path);

        assert!(config.nerd_fonts);
        assert_eq!(config.includes, vec!["extra.toml", "more.toml"]);

        fs::remove_file(&path).ok();
    }

    #[test]
    fn load_config_from_file_invalid_toml_returns_default() {
        let path = std::env::temp_dir().join("cs_test_config_invalid.toml");
        fs::write(&path, "this is not [[valid toml}}").unwrap();

        let config = load_config_from_file(&path);

        assert!(!config.nerd_fonts);
        assert!(config.includes.is_empty());

        fs::remove_file(&path).ok();
    }

    #[test]
    fn load_snippet_from_empty_file_returns_empty() {
        let dir = std::env::temp_dir().join("cs_test_empty");
        let path = dir.join("empty.toml");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path, "").unwrap();

        let result = load_snippets_from_file(&path);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());

        fs::remove_file(&path).ok();
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn load_snippet_from_missing_file_returns_empty() {
        let path = std::env::temp_dir().join("cs_test_nonexistent");
        let result = load_snippets_from_file(&path);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn load_snippet_from_file_sets_source() {
        let dir = std::env::temp_dir().join("cs_test_source");
        let path = dir.join("test.toml");
        fs::create_dir_all(&dir).unwrap();

        let content = r#"
[[snippet]]
cmd = "echo hello"
description = "greet"
tags = ["test"]
os = "linux"
"#;
        fs::write(&path, content).unwrap();

        let result = load_snippets_from_file(&path).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].cmd, "echo hello");
        assert_eq!(result[0].source, path);

        fs::remove_file(&path).ok();
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_snippets_groups_by_source_file() {
        let dir = std::env::temp_dir().join("cs_test_save_groups");
        let path1 = dir.join("file1.toml");
        let path2 = dir.join("file2.toml");
        fs::create_dir_all(&dir).unwrap();
        fs::write(&path1, "").unwrap();
        fs::write(&path2, "").unwrap();

        let snippets = vec![
            make_snippet_with_source("cmd1", "first", vec![], Os::Linux, path1.clone()),
            make_snippet_with_source("cmd2", "second", vec![], Os::Linux, path1.clone()),
            make_snippet_with_source("cmd3", "third", vec![], Os::Macos, path2.clone()),
        ];
        let result = save_snippets(&snippets);

        assert!(result.is_ok());

        let content1 = fs::read_to_string(&path1).unwrap();

        assert!(content1.contains("cmd1"));
        assert!(content1.contains("cmd2"));
        assert!(content1.contains("[[snippet]]"));

        let content2 = fs::read_to_string(&path2).unwrap();

        assert!(content2.contains("cmd3"));

        fs::remove_file(&path1).ok();
        fs::remove_file(&path2).ok();
        fs::remove_dir_all(&dir).ok();
    }
}
