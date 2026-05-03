use crate::selector;
use crate::storage;
use crate::utils::format;
use crate::utils::icons::{EMOJI_DELETE, EMOJI_SUCCESS};
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};

pub fn run(query: &str, all_os: bool) -> Result<(), String> {
    let filtered = storage::filter_snippets(query, all_os)?;
    if filtered.is_none() {
        return Ok(());
    }
    let filtered = filtered.unwrap();

    let display_snippets: Vec<_> = filtered.iter().map(|(_, s)| (*s).clone()).collect();
    let idx = selector::select_snippet(&display_snippets).ok_or("Selection cancelled")?;
    let (original_idx, selected) = &filtered[idx];

    println!();
    println!(
        "{}",
        style(format!("{}  Snippet to delete:", EMOJI_DELETE))
            .red()
            .bold()
    );
    format::print_snippet(selected);
    println!();

    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Delete this snippet?")
        .default(false)
        .interact()
        .map_err(|e| format!("Failed to read confirmation: {}", e))?;

    if confirm {
        let all_snippets = storage::load_snippets().to_vec();
        storage::delete_snippet(&all_snippets, *original_idx)?;
        println!(
            "{}",
            style(format!("{} Snippet deleted.", EMOJI_SUCCESS)).green()
        );
    } else {
        println!("{}", style("Deletion cancelled.").dim());
    }

    Ok(())
}
