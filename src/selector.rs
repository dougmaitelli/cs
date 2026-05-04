use crate::snippet::Snippet;
use crate::utils::format;
use dialoguer::{theme::ColorfulTheme, FuzzySelect};
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

static NO_FZF: AtomicBool = AtomicBool::new(false);

const DIALOGUER_MAX_ITEMS: usize = 100;

pub(crate) fn set_no_fzf(value: bool) {
    NO_FZF.store(value, Ordering::Relaxed);
}

fn has_fzf() -> bool {
    Command::new("fzf")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn use_fzf() -> bool {
    !NO_FZF.load(Ordering::Relaxed) && has_fzf()
}

fn select_strings_with_fzf(items: &[String]) -> Option<String> {
    let mut child = Command::new("fzf")
        .arg("--ansi")
        .arg("--tac")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .ok()?;

    {
        let stdin = child.stdin.as_mut()?;
        for item in items {
            writeln!(stdin, "{}", item).ok()?;
        }
    }

    let output = child.wait_with_output().ok()?;
    if !output.status.success() {
        return None;
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected.is_empty() {
        None
    } else {
        Some(selected)
    }
}

pub(crate) fn select_strings_with_dialoguer(items: &[String], prompt: &str, default_idx: usize) -> Option<usize> {
    let display_items: Vec<&str> = items
        .iter()
        .take(DIALOGUER_MAX_ITEMS)
        .map(|s| s.as_str())
        .collect();

    FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .highlight_matches(false)
        .items(&display_items)
        .default(default_idx)
        .interact_opt()
        .ok()
        .flatten()
}

fn select_snippet_with_fzf(snippets: &[Snippet]) -> Option<usize> {
    let items: Vec<String> = format::format_rows(snippets);
    let indexed: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(i, item)| format!("{}: {}", i, item))
        .collect();

    let selected = select_strings_with_fzf(&indexed)?;
    selected.split(':').next()?.trim().parse().ok()
}

fn select_snippet_with_dialoguer(snippets: &[Snippet]) -> Option<usize> {
    let items:  Vec<String> = format::format_rows(snippets);
    select_strings_with_dialoguer(&items, "Select a snippet", 0)
}

pub(crate) fn select_strings(items: &[String]) -> Option<String> {
    if use_fzf() {
        select_strings_with_fzf(items)
    } else {
        let idx = select_strings_with_dialoguer(items, "Select a command", 0)?;
        Some(items[idx].clone())
    }
}

pub(crate) fn select_snippet(snippets: &[Snippet]) -> Option<usize> {
    if use_fzf() {
        select_snippet_with_fzf(snippets)
    } else {
        select_snippet_with_dialoguer(snippets)
    }
}

// ----------------------------------------------------------------------------
// TESTS
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn fzf_flag_is_false_by_default() {
        assert!(!NO_FZF.load(Ordering::Relaxed));
    }

    #[test]
    fn set_no_fzf_stores_value() {
        NO_FZF.store(true, Ordering::Relaxed);
        assert!(NO_FZF.load(Ordering::Relaxed));

        NO_FZF.store(false, Ordering::Relaxed);
        assert!(!NO_FZF.load(Ordering::Relaxed));
    }
}
