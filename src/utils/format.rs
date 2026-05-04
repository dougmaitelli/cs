use crate::snippet::Snippet;
use console::style;

fn column_widths(snippets: &[Snippet]) -> (usize, usize, usize) {
    let cmd_width = snippets
        .iter()
        .map(|s| s.cmd.len())
        .max()
        .unwrap_or(10)
        .max(7);
    let desc_width = snippets
        .iter()
        .map(|s| s.description.len())
        .max()
        .unwrap_or(10)
        .max(11);
    let tags_width = snippets
        .iter()
        .map(|s| {
            if s.tags.is_empty() {
                2
            } else {
                s.tags.join(", ").len()
            }
        })
        .max()
        .unwrap_or(4)
        .max(4);

    (cmd_width, desc_width, tags_width)
}

pub(crate) fn print_header(snippets: &[Snippet], indent: &str) {
    let (cmd_width, desc_width, tags_width) = column_widths(snippets);

    println!(
        "{}{}  {:<cmd_width$}  {:<desc_width$}  {:<tags_width$}  {}",
        indent,
        style("OS").dim(),
        style("Command").dim(),
        style("Description").dim(),
        style("Tags").dim(),
        style("OS").dim(),
        cmd_width = cmd_width,
        desc_width = desc_width,
        tags_width = tags_width,
    );

    println!(
        "{}{}  {:<cmd_width$}  {:<desc_width$}  {:<tags_width$}  {}",
        indent,
        style("──").dim(),
        style("─".repeat(cmd_width)).dim(),
        style("─".repeat(desc_width)).dim(),
        style("─".repeat(tags_width)).dim(),
        style("────────").dim(),
        cmd_width = cmd_width,
        desc_width = desc_width,
        tags_width = tags_width,
    );
}

fn format_row(snippet: &Snippet, cmd_width: usize, desc_width: usize, tags_width: usize) -> String {
    let tags_str = if snippet.tags.is_empty() {
        "--".to_string()
    } else {
        snippet.tags.join(", ")
    };

    format!(
        "{}  {:<cmd_width$}  {:<desc_width$}  {:<tags_width$}  {}",
        snippet.os.icon(),
        style(&snippet.cmd).cyan().bold(),
        style(&snippet.description).white(),
        style(&tags_str).dim(),
        style(format!("({})", snippet.os)).magenta(),
        cmd_width = cmd_width,
        desc_width = desc_width,
        tags_width = tags_width,
    )
}

pub(crate) fn format_rows(snippets: &[Snippet]) -> Vec<String> {
    let (cmd_width, desc_width, tags_width) = column_widths(snippets);

    snippets
        .iter()
        .map(|s| format_row(s, cmd_width, desc_width, tags_width))
        .collect()
}

pub(crate) fn print_snippet(snippet: &Snippet) {
    println!(
        "   {}  {}",
        style("Command:").dim(),
        style(&snippet.cmd).cyan()
    );
    println!(
        "   {}  {}",
        style("Description:").dim(),
        &snippet.description
    );
    println!(
        "   {}  {}",
        style("Tags:").dim(),
        if snippet.tags.is_empty() {
            style("(none)".to_string()).dim().to_string()
        } else {
            snippet.tags.join(", ")
        }
    );
    println!(
        "   {}  {}",
        style("OS:").dim(),
        style(&snippet.os).magenta()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::os::Os;
    use std::path::PathBuf;

    fn make_snippet(cmd: &str, description: &str, tags: Vec<&str>) -> Snippet {
        Snippet {
            cmd: cmd.to_string(),
            description: description.to_string(),
            tags: tags.into_iter().map(String::from).collect(),
            os: Os::Linux,
            source: PathBuf::new(),
        }
    }

    #[test]
    fn column_widths_empty_snippets() {
        let snippets: Vec<Snippet> = vec![];
        let (cmd_w, desc_w, tags_w) = column_widths(&snippets);
        assert_eq!(cmd_w, 10);
        assert_eq!(desc_w, 11);
        assert_eq!(tags_w, 4);
    }

    #[test]
    fn column_widths_single_snippet() {
        let snippets = vec![make_snippet("ls", "list files", vec![])];
        let (cmd_w, desc_w, tags_w) = column_widths(&snippets);
        assert_eq!(cmd_w, 7);
        assert_eq!(desc_w, 11);
        assert_eq!(tags_w, 4);
    }

    #[test]
    fn column_widths_with_long_cmd() {
        let snippets = vec![make_snippet(
            "cargo build --release",
            "build release",
            vec![],
        )];
        let (cmd_w, _, _) = column_widths(&snippets);
        assert_eq!(cmd_w, 21);
    }

    #[test]
    fn column_widths_with_tags() {
        let snippets = vec![make_snippet("ls", "list", vec!["tag1", "tag2"])];
        let (_, _, tags_w) = column_widths(&snippets);
        assert_eq!(tags_w, 10);
    }

    #[test]
    fn column_widths_multiple_snippets() {
        let snippets = vec![
            make_snippet("ls", "short", vec![]),
            make_snippet(
                "very-long-command-name",
                "a much longer description here",
                vec!["a", "longer-tag-list"],
            ),
        ];
        let (cmd_w, desc_w, tags_w) = column_widths(&snippets);
        assert_eq!(cmd_w, 22);
        assert_eq!(desc_w, 30);
        assert_eq!(tags_w, 18);
    }

    #[test]
    fn format_row_single_snippet() {
        let snippet = make_snippet("ls -la", "list files", vec![]);
        let row = format_row(&snippet, 10, 11, 4);
        assert!(row.contains("ls -la"));
        assert!(row.contains("list files"));
        assert!(row.contains("--"));
    }

    #[test]
    fn format_row_with_tags() {
        let snippet = make_snippet("git push", "push changes", vec!["git", "network"]);
        let row = format_row(&snippet, 10, 13, 11);
        assert!(row.contains("git push"));
        assert!(row.contains("push changes"));
        assert!(row.contains("git, network"));
    }

    #[test]
    fn format_rows_empty() {
        let snippets: Vec<Snippet> = vec![];
        let rows = format_rows(&snippets);
        assert!(rows.is_empty());
    }

    #[test]
    fn format_rows_multiple() {
        let snippets = vec![
            make_snippet("ls", "list", vec![]),
            make_snippet("grep", "search", vec!["text"]),
        ];
        let rows = format_rows(&snippets);
        assert_eq!(rows.len(), 2);
        assert!(rows[0].contains("ls"));
        assert!(rows[1].contains("grep"));
    }

    #[test]
    fn format_rows_consistent_widths() {
        let snippets = vec![
            make_snippet("ls", "short", vec![]),
            make_snippet(
                "very-long-command",
                "a much longer description",
                vec!["a", "b"],
            ),
        ];
        let rows = format_rows(&snippets);
        for row in &rows {
            assert_eq!(row.len(), rows[0].len());
        }
    }
}
