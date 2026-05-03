mod commands;
mod os;
mod selector;
mod snippet;
mod storage;
mod utils;

use clap::{Parser, Subcommand};
use console::style;

#[derive(Parser)]
#[command(name = "cs")]
#[command(about = "CLI snippet manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Search query (when no subcommand)
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,

    /// Show snippets for all operating systems
    #[arg(long, global = true)]
    all_os: bool,

    /// Disable fzf and use built-in selector
    #[arg(long, global = true)]
    no_fzf: bool,
}

#[derive(Subcommand)]
enum Commands {
    Add,
    Import,
    List {
        #[arg(long)]
        all_os: bool,
    },
    Delete {
        #[arg(trailing_var_arg = true)]
        query: Vec<String>,

        #[arg(long)]
        all_os: bool,
    },
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--version".to_string()) {
        let version = option_env!("VERSION").unwrap_or("dev");

        println!(
            "{} {} {}",
            style("✨").green().dim(),
            style("cs").bold(),
            style(version).yellow()
        );

        std::process::exit(0);
    }

    let cli = Cli::parse();
    selector::set_no_fzf(cli.no_fzf);

    if let Err(e) = storage::init_store(&storage::config_path()) {
        eprintln!("Error loading config: {}", e);
        std::process::exit(1);
    }

    let result = match cli.command {
        Some(Commands::Add) => commands::add::run(),
        Some(Commands::Import) => commands::import::run(),
        Some(Commands::List { all_os }) => commands::list::run(all_os),
        Some(Commands::Delete { query, all_os }) => {
            let query = query.join(" ");
            commands::delete::run(&query, all_os)
        }
        _ => {
            let query = cli.query.join(" ");
            commands::search::run(&query, cli.all_os)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

// ----------------------------------------------------------------------------
// TESTS
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::Cli;
    use crate::Commands;
    use clap::Parser;

    fn parse<const N: usize>(args: &[&str; N]) -> Cli {
        Cli::try_parse_from(args).unwrap()
    }

    #[test]
    fn no_args() {
        let cli = parse(&["cs"]);

        assert!(cli.command.is_none());
        assert!(cli.query.is_empty());
        assert!(!cli.all_os);
        assert!(!cli.no_fzf);
    }

    #[test]
    fn bare_search_query() {
        let cli = parse(&["cs", "hello"]);

        assert!(cli.command.is_none());
        assert_eq!(cli.query, vec!["hello"]);
    }

    #[test]
    fn bare_search_multiple_words() {
        let cli = parse(&["cs", "git", "push"]);

        assert!(cli.command.is_none());
        assert_eq!(cli.query, vec!["git", "push"]);
    }

    #[test]
    fn add_command() {
        let cli = parse(&["cs", "add"]);

        assert!(matches!(cli.command, Some(Commands::Add)));
    }

    #[test]
    fn import_command() {
        let cli = parse(&["cs", "import"]);

        assert!(matches!(cli.command, Some(Commands::Import)));
    }

    #[test]
    fn list_command() {
        let cli = parse(&["cs", "list"]);

        assert!(matches!(
            cli.command,
            Some(Commands::List { all_os: false })
        ));
    }

    #[test]
    fn list_command_all_os() {
        let cli = parse(&["cs", "list", "--all-os"]);

        assert!(matches!(cli.command, Some(Commands::List { all_os: true })));
    }

    #[test]
    fn delete_command() {
        let cli = parse(&["cs", "delete", "test"]);

        assert!(
            matches!(cli.command, Some(Commands::Delete { ref query, all_os: false }) if query == &vec!["test".to_string()])
        );
    }

    #[test]
    fn delete_command_with_query_and_all_os() {
        let cli = parse(&["cs", "--all-os", "delete", "hello"]);

        assert!(cli.all_os);
        assert!(
            matches!(cli.command, Some(Commands::Delete { ref query, all_os: true }) if query == &vec!["hello".to_string()])
        );
    }

    #[test]
    fn global_no_fzf_flag() {
        let cli = parse(&["cs", "--no-fzf", "list"]);

        assert!(cli.no_fzf);
    }

    #[test]
    fn global_all_os_flag() {
        let cli = parse(&["cs", "--all-os", "list"]);

        assert!(cli.all_os);
    }
}
