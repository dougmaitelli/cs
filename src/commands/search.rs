use crate::selector;
use crate::storage;
use console::style;
use rustyline::DefaultEditor;
use std::process::Command;

pub(crate) fn run(query: &str, all_os: bool) -> Result<(), String> {
    let filtered = storage::filter_snippets(query, all_os)?;
    if filtered.is_none() {
        return Ok(());
    }
    let filtered = filtered.unwrap();

    let selected = if filtered.len() == 1 {
        &filtered[0].1
    } else {
        let display_snippets: Vec<_> = filtered.iter().map(|(_, s)| (*s).clone()).collect();
        let idx = selector::select_snippet(&display_snippets).ok_or("Selection cancelled")?;
        &filtered[idx].1
    };

    println!();
    let cmd = edit_and_execute(&selected.cmd)?;
    println!();

    execute_command(&cmd)
}

fn edit_and_execute(initial: &str) -> Result<String, String> {
    let mut rl = DefaultEditor::new().map_err(|e| format!("Failed to initialize editor: {}", e))?;
    let prompt = format!("{} ", style("$").green().bold());
    let line = rl
        .readline_with_initial(&prompt, (initial, ""))
        .map_err(|e| format!("Failed to read line: {}", e))?;

    Ok(line)
}

fn execute_command(cmd: &str) -> Result<(), String> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let status = Command::new(&shell)
        .arg("-c")
        .arg(cmd)
        .status()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if !status.success() {
        if let Some(code) = status.code() {
            std::process::exit(code);
        }
    }

    Ok(())
}
