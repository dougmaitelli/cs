use crate::selector;
use crate::snippet::{Os, Snippet};
use crate::storage;
use crate::utils::icons::{EMOJI_ADD, EMOJI_EMPTY, EMOJI_SUCCESS};
use crate::utils::os_detect;
use console::style;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input};
use std::path::PathBuf;
use strum::IntoEnumIterator;

fn get_shell_history() -> Result<Vec<String>, String> {
    let shell = std::env::var("SHELL").unwrap_or_default();
    let home = dirs::home_dir().ok_or("Could not find home directory")?;

    let history_file = if shell.contains("zsh") {
        home.join(".zsh_history")
    } else if shell.contains("bash") {
        home.join(".bash_history")
    } else if shell.contains("fish") {
        home.join(".local/share/fish/fish_history")
    } else {
        return Err(format!("Unsupported shell: {}", shell));
    };

    let content = std::fs::read_to_string(&history_file)
        .map_err(|e| format!("Failed to read history file: {}", e))?;

    let lines: Vec<String> = if shell.contains("zsh") {
        content
            .lines()
            .filter_map(|line| {
                if line.starts_with(':') {
                    line.split_once(';').map(|(_, s)| s.to_string())
                } else {
                    Some(line.to_string())
                }
            })
            .filter(|s| !s.is_empty())
            .collect()
    } else if shell.contains("fish") {
        content
            .lines()
            .filter_map(|line| {
                if line.starts_with("- cmd: ") {
                    Some(line.trim_start_matches("- cmd: ").to_string())
                } else {
                    None
                }
            })
            .collect()
    } else {
        content
            .lines()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect()
    };

    let mut unique: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for line in lines.into_iter().rev() {
        if seen.insert(line.clone()) {
            unique.push(line);
        }
    }

    Ok(unique)
}

pub(crate) fn run() -> Result<(), String> {
    let current_os = os_detect::detect_os();
    let history = get_shell_history()?;

    if history.is_empty() {
        println!(
            "{}",
            style(format!("{} No command history found.", EMOJI_EMPTY)).yellow()
        );
        return Ok(());
    }

    let selected = selector::select_strings(&history);

    let cmd = match selected {
        Some(c) => c,
        _ => {
            println!("{}", style("Selection cancelled.").dim());
            return Ok(());
        }
    };

    println!();
    println!(
        "{}",
        style(format!("{} Import command", EMOJI_ADD))
            .green()
            .bold()
    );
    println!("  {} {}", style("Command:").dim(), style(&cmd).cyan());
    println!();

    let description: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Description")
        .interact_text()
        .map_err(|e| format!("Failed to read description: {}", e))?;

    let tags_input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Tags (comma-separated)")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| format!("Failed to read tags: {}", e))?;

    let tags: Vec<String> = tags_input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let os_options: Vec<&str> = Os::iter().map(|o| o.into()).collect();
    let default_idx = Os::iter().position(|o| o == current_os).unwrap_or(0);

    let os_idx = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("OS")
        .items(&os_options)
        .default(default_idx)
        .interact()
        .map_err(|e| format!("Failed to read OS: {}", e))?;

    let os = Os::iter().nth(os_idx).ok_or("No OS available")?;

    let target = storage::select_target_file()?;

    let snippet = Snippet {
        cmd,
        description,
        tags,
        os,
        source: PathBuf::new(),
    };

    storage::add_snippet(snippet.clone(), target)?;

    println!();
    println!(
        "{} {}",
        style(EMOJI_SUCCESS).green().bold(),
        style("Snippet imported:").green()
    );
    println!("  {}", snippet.display_line_with_os());
    Ok(())
}
