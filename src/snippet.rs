pub use crate::os::Os;
use console::style;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub cmd: String,
    pub description: String,
    pub tags: Vec<String>,
    pub os: Os,
    #[serde(skip)]
    pub source: PathBuf,
}

impl Snippet {
    pub fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();

        self.cmd.to_lowercase().contains(&query)
            || self.description.to_lowercase().contains(&query)
            || Into::<&str>::into(self.os).contains(&query)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&query))
    }

    pub fn matches_os(&self, current_os: &Os) -> bool {
        self.os == Os::Any || self.os == *current_os
    }

    pub fn display_line_with_os(&self) -> String {
        let os_icon = self.os.icon();

        let tags = if self.tags.is_empty() {
            String::new()
        } else {
            format!(" {}", style(format!("[{}]", self.tags.join(", "))).dim())
        };

        format!(
            "{} {} {} {}{} {}",
            os_icon,
            style(&self.cmd).cyan().bold(),
            style("—").dim(),
            style(&self.description).white(),
            tags,
            style(format!("({})", self.os)).magenta()
        )
    }
}

// ----------------------------------------------------------------------------
// TESTS
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snippet(cmd: &str, description: &str, tags: Vec<&str>, os: Os) -> Snippet {
        Snippet {
            cmd: cmd.to_string(),
            description: description.to_string(),
            tags: tags.into_iter().map(String::from).collect(),
            os,
            source: PathBuf::new(),
        }
    }

    #[test]
    fn matches_by_cmd() {
        let s = make_snippet("ls -la", "list files", vec![], Os::Linux);

        assert!(s.matches("ls"));
        assert!(s.matches("ls -la"));
        assert!(s.matches("LS"));
        assert!(s.matches("LS -LA"));
    }

    #[test]
    fn matches_by_description() {
        let s = make_snippet(
            "ls -la",
            "list files in current directory",
            vec![],
            Os::Linux,
        );

        assert!(s.matches("list files"));
        assert!(s.matches("LIST FILES"));
        assert!(s.matches("current directory"));
    }

    #[test]
    fn matches_by_os() {
        let s = make_snippet("ls", "list files", vec![], Os::Ubuntu);

        assert!(s.matches("ubuntu"));
        assert!(s.matches("UBUNTU"));
    }

    #[test]
    fn matches_by_tag() {
        let s = make_snippet(
            "docker run",
            "run container",
            vec!["docker", "containers"],
            Os::Linux,
        );

        assert!(s.matches("docker"));
        assert!(s.matches("containers"));
        assert!(s.matches("DOCKER"));
    }

    #[test]
    fn matches_no_match() {
        let s = make_snippet("ls -la", "list files", vec!["system"], Os::Linux);

        assert!(!s.matches("git"));
        assert!(!s.matches("cargo"));
        assert!(!s.matches("nonexistent"));
    }

    #[test]
    fn matches_any_os() {
        let s = make_snippet("echo hello", "print hello", vec![], Os::Any);

        assert!(s.matches("any"));
        assert!(s.matches("Hello"));
    }

    #[test]
    fn matches_os_current() {
        let s = make_snippet("ls", "list", vec![], Os::Linux);

        assert!(s.matches_os(&Os::Linux));
    }

    #[test]
    fn matches_os_different() {
        let s = make_snippet("ls", "list", vec![], Os::Linux);

        assert!(!s.matches_os(&Os::Macos));
        assert!(!s.matches_os(&Os::Windows));
    }

    #[test]
    fn matches_os_any_matches_all() {
        let s = make_snippet("echo", "print", vec![], Os::Any);

        assert!(s.matches_os(&Os::Linux));
        assert!(s.matches_os(&Os::Macos));
        assert!(s.matches_os(&Os::Windows));
        assert!(s.matches_os(&Os::Ubuntu));
        assert!(s.matches_os(&Os::Any));
    }
}
