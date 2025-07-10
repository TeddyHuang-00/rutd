use clap::{Parser, Subcommand};
use clap_complete::ArgValueCompleter;
use rutd_core::{MergeStrategy, Priority, SortOptions};

use super::FilterOptions;
use crate::{completer, parser};

/// RuTD - A Rust based To-Do list manager for your rushing to-dos
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Verbosity level
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Add a new task
    ///
    /// Add a new task to the to-do list, supporting specification of
    /// description, priority, scope, and type
    #[command(visible_aliases = ["a"])]
    Add {
        /// Task description
        description: String,

        /// Task priority
        #[arg(
            short, long,
            default_value = Priority::Normal.as_ref(),
            add = ArgValueCompleter::new(completer::complete_priority)
        )]
        priority: Priority,

        /// Task scope (project name)
        #[arg(
            short = 's', long = "scope",
            value_name = "SCOPE",
            add = ArgValueCompleter::new(completer::complete_scope)
        )]
        task_scope: Option<String>,

        /// Task type (e.g., feat, fix, other, etc.)
        #[arg(
            short = 't', long = "type",
            value_name = "TYPE",
            add = ArgValueCompleter::new(completer::complete_type)
        )]
        task_type: Option<String>,
    },
    /// List tasks
    ///
    /// List tasks in the to-do list, supporting filtering by priority, scope,
    /// type, and status
    #[command(visible_aliases = ["l"])]
    List {
        /// Filter options
        #[command(flatten)]
        filter: FilterOptions,

        /// Sort options
        #[arg(
            short = 'o', long = "sort",
            allow_hyphen_values = true,
            value_parser = parser::parse_sort_options,
            add = ArgValueCompleter::new(completer::complete_sort_options)
        )]
        sort: Option<SortOptions>,

        /// Show statistics (counts, total time spent)
        #[arg(long)]
        stats: bool,
    },
    /// Mark task as completed
    ///
    /// Mark the task with the specified ID as completed
    #[command(visible_aliases = ["d", "f"])]
    Done {
        /// Task ID
        #[arg(add = ArgValueCompleter::new(completer::complete_id))]
        id: String,
    },
    /// Edit task description
    ///
    /// Edit the description of the task with the specified ID using the default
    /// editor
    #[command(visible_aliases = ["e"])]
    Edit {
        /// Task ID
        #[arg(add = ArgValueCompleter::new(completer::complete_id))]
        id: String,
    },
    /// Start working on a task
    ///
    /// Mark the task with the specified ID as in progress and start time
    /// tracking
    #[command(visible_aliases = ["s"])]
    Start {
        /// Task ID
        #[arg(add = ArgValueCompleter::new(completer::complete_id))]
        id: String,
    },
    /// Stop working on active task
    ///
    /// Pause time tracking for the active task
    #[command(visible_aliases = ["p"])]
    Stop {},
    /// Abort a task
    ///
    /// Mark the task with the specified ID as aborted
    #[command(visible_aliases = ["x", "c"])]
    Abort {
        /// Task ID, if not specified, abort the active task
        #[arg(add = ArgValueCompleter::new(completer::complete_id))]
        id: Option<String>,
    },
    /// Clean tasks
    ///
    /// Remove tasks based on filters
    #[command(visible_aliases = ["purge", "delete", "rm"])]
    Clean {
        /// Filter options
        #[command(flatten)]
        filter: FilterOptions,

        /// Confirm deletion without prompting
        #[arg(long)]
        force: bool,
    },
    /// Sync with remote repository
    ///
    /// Fetch, pull and push changes to the remote repository
    #[command(visible_aliases = ["y", "u"])]
    Sync {
        /// Conflict resolution preference when merging
        #[arg(
            short, long,
            default_value = MergeStrategy::None.as_ref(),
            add = ArgValueCompleter::new(completer::complete_merge_strategy)
        )]
        prefer: MergeStrategy,
    },
    /// Clone a remote repository
    ///
    /// Clone a remote repository to the local tasks directory
    #[command(visible_aliases = ["pull"])]
    Clone {
        /// Remote repository URL to clone
        #[arg(value_hint = clap::ValueHint::Url)]
        url: String,
    },
    /// Manage configuration
    ///
    /// Get, set, or list configuration values
    #[command(visible_aliases = ["cfg"])]
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., "git.username", "path.root_dir")
        #[arg(add = ArgValueCompleter::new(completer::complete_config_key))]
        key: String,
    },
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., "git.username", "path.root_dir")
        #[arg(add = ArgValueCompleter::new(completer::complete_config_key))]
        key: String,
        /// Configuration value
        value: String,
    },
    /// Remove a configuration value
    Unset {
        /// Configuration key (e.g., "git.username", "path.root_dir")
        #[arg(add = ArgValueCompleter::new(completer::complete_config_key))]
        key: String,
    },
    /// Show all configuration values
    Show,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn test_cli_command_verifies() {
        // This test verifies that the Cli struct's command specification is valid
        Cli::command().debug_assert();
    }

    #[test]
    fn test_cli_command_parse() {
        // Test basic command parsing without arguments
        let result = Cli::try_parse_from(["rutd", "list"]);
        assert!(result.is_ok());

        let cli = result.unwrap();
        match cli.command {
            Commands::List {
                filter,
                sort,
                stats,
            } => {
                assert!(sort.is_none());
                assert!(!stats);
                // Default filter should be empty
                assert!(filter.priority.is_none());
                assert!(filter.task_scope.is_none());
                assert!(filter.task_type.is_none());
                assert!(filter.status.is_none());
            }
            _ => panic!("Should have parsed as list command"),
        }
    }

    #[test]
    fn test_add_command() {
        // Test the Add command with various arguments
        let result = Cli::try_parse_from([
            "rutd",
            "add",
            "Test description",
            "--priority",
            "high",
            "--scope",
            "test-project",
            "--type",
            "feature",
        ]);

        assert!(result.is_ok());

        let cli = result.unwrap();
        match cli.command {
            Commands::Add {
                description,
                priority,
                task_scope,
                task_type,
            } => {
                assert_eq!(description, "Test description");
                assert_eq!(priority, Priority::High);
                assert_eq!(task_scope, Some("test-project".to_string()));
                assert_eq!(task_type, Some("feature".to_string()));
            }
            _ => panic!("Should have parsed as add command"),
        }
    }

    #[test]
    fn test_done_command() {
        // Test the Done command
        let result = Cli::try_parse_from(["rutd", "done", "1a2b3c"]);

        assert!(result.is_ok());

        let cli = result.unwrap();
        match cli.command {
            Commands::Done { id } => {
                assert_eq!(id, "1a2b3c");
            }
            _ => panic!("Should have parsed as done command"),
        }
    }

    #[test]
    fn test_start_command() {
        // Test the Start command
        let result = Cli::try_parse_from(["rutd", "start", "1a2b3c"]);

        assert!(result.is_ok());

        let cli = result.unwrap();
        match cli.command {
            Commands::Start { id } => {
                assert_eq!(id, "1a2b3c");
            }
            _ => panic!("Should have parsed as start command"),
        }
    }

    #[test]
    fn test_stop_command() {
        // Test the Stop command (no arguments)
        let result = Cli::try_parse_from(["rutd", "stop"]);

        assert!(result.is_ok());

        let cli = result.unwrap();
        match cli.command {
            Commands::Stop {} => (),
            _ => panic!("Should have parsed as stop command"),
        }
    }

    #[test]
    fn test_abort_command_with_id() {
        // Test the Abort command with ID
        let result = Cli::try_parse_from(["rutd", "abort", "1a2b3c"]);

        assert!(result.is_ok());

        let cli = result.unwrap();
        match cli.command {
            Commands::Abort { id } => {
                assert_eq!(id, Some("1a2b3c".to_string()));
            }
            _ => panic!("Should have parsed as abort command"),
        }
    }

    #[test]
    fn test_abort_command_without_id() {
        // Test the Abort command without ID (should abort active task)
        let result = Cli::try_parse_from(["rutd", "abort"]);

        assert!(result.is_ok());

        let cli = result.unwrap();
        match cli.command {
            Commands::Abort { id } => {
                assert_eq!(id, None);
            }
            _ => panic!("Should have parsed as abort command"),
        }
    }

    #[test]
    fn test_clean_command() {
        // Test the Clean command with force flag
        let result = Cli::try_parse_from(["rutd", "clean", "--force"]);

        assert!(result.is_ok());

        let cli = result.unwrap();
        match cli.command {
            Commands::Clean { filter, force } => {
                assert!(force);
                // Default filter should be empty
                assert!(filter.priority.is_none());
                assert!(filter.task_scope.is_none());
                assert!(filter.task_type.is_none());
                assert!(filter.status.is_none());
            }
            _ => panic!("Should have parsed as clean command"),
        }
    }

    #[test]
    fn test_sync_command() {
        // Test the Sync command with prefer option
        let result = Cli::try_parse_from(["rutd", "sync", "--prefer", "local"]);

        assert!(result.is_ok());

        let cli = result.unwrap();
        match cli.command {
            Commands::Sync { prefer } => {
                assert_eq!(prefer, MergeStrategy::Local);
            }
            _ => panic!("Should have parsed as sync command"),
        }
    }

    #[test]
    fn test_clone_command() {
        // Test the Clone command
        let result = Cli::try_parse_from(["rutd", "clone", "https://example.com/repo.git"]);

        assert!(result.is_ok());

        let cli = result.unwrap();
        match cli.command {
            Commands::Clone { url } => {
                assert_eq!(url, "https://example.com/repo.git");
            }
            _ => panic!("Should have parsed as clone command"),
        }
    }

    #[test]
    fn test_verbosity_flag() {
        // Test verbosity flag with different counts
        let result = Cli::try_parse_from(["rutd", "-vv", "list"]);

        assert!(result.is_ok());

        let cli = result.unwrap();
        assert_eq!(cli.verbose, 2);

        // Test with long version
        let result = Cli::try_parse_from(["rutd", "--verbose", "--verbose", "list"]);

        assert!(result.is_ok());

        let cli = result.unwrap();
        assert_eq!(cli.verbose, 2);
    }
}
