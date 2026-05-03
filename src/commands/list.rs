use crate::storage;
use crate::utils::format;
use crate::utils::icons::{EMOJI_EMPTY, EMOJI_LIST, EMOJI_NOT_FOUND};
use crate::utils::os_detect;
use console::style;

pub fn run(all_os: bool) -> Result<(), String> {
    let current_os = os_detect::detect_os();
    let snippets = storage::load_snippets();

    if snippets.is_empty() {
        println!(
            "{}",
            style(format!(
                "{} No snippets yet. Use `cs add` to create one.",
                EMOJI_EMPTY
            ))
            .yellow()
        );
        return Ok(());
    }

    let filtered: Vec<_> = if all_os {
        snippets.to_vec()
    } else {
        snippets
            .iter()
            .filter(|s| s.matches_os(&current_os))
            .cloned()
            .collect()
    };

    if filtered.is_empty() {
        println!(
            "{}",
            style(format!(
                "{} No snippets found for {}. Use --all-os to see all.",
                EMOJI_NOT_FOUND, current_os
            ))
            .yellow()
        );
        return Ok(());
    }

    println!(
        "{} {}",
        style(EMOJI_LIST).dim(),
        style(format!("{} snippet(s)", filtered.len())).dim()
    );
    println!();

    format::print_header(&filtered, "  ");
    for row in format::format_rows(&filtered) {
        println!("  {}", row);
    }

    Ok(())
}
