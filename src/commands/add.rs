use crate::snippet::{Os, Snippet};
use crate::storage;
use crate::utils::icons::{EMOJI_ADD, EMOJI_SUCCESS};
use crate::utils::os_detect;
use console::style;
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input};
use std::path::PathBuf;
use strum::IntoEnumIterator;

pub fn run() -> Result<(), String> {
    println!(
        "{}",
        style(format!("{} Add new snippet", EMOJI_ADD))
            .green()
            .bold()
    );
    println!();

    let cmd: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Command")
        .interact_text()
        .map_err(|e| format!("Failed to read command: {}", e))?;

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

    let current_os = os_detect::detect_os();
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
        style("Snippet added:").green()
    );
    println!("  {}", snippet.display_line_with_os());

    Ok(())
}
