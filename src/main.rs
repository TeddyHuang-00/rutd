mod cli;
mod config;
mod display;
mod git;
mod logging;
mod task;

use std::process::ExitCode;

use clap::Parser;
use cli::Cli;
use config::Config;
use display::DisplayManager;
use log::{debug, trace};
use task::TaskManager;

fn main() -> ExitCode {
    // Parse command line arguments
    let cli = Cli::parse();

    // Get configuration from environment variables
    let Ok(config) =
        Config::new().inspect_err(|e| eprintln!("Failed to load configuration: {}", e))
    else {
        return ExitCode::FAILURE;
    };

    // Initialize logging system
    if let Err(e) = logging::init_logger(
        cli.verbose,
        config.path.log_file(),
        config.general.max_log_history,
    ) {
        eprintln!("{}", e);
        return ExitCode::FAILURE;
    }

    trace!("Received cli args: {:?}", cli);
    trace!("Loaded configuration: {:?}", config);

    let path_config = config.path;
    let git_config = config.git;

    // Create a display manager
    let display_manager = DisplayManager::default();

    // Build the task manager
    let task_manager = TaskManager::new(path_config, git_config);

    // Handle different commands
    match &cli.command {
        cli::commands::Commands::Add {
            description,
            priority,
            scope,
            task_type,
        } => {
            trace!("Add task command");
            debug!("Add task: {}", description);
            debug!("Priority: {}", priority);
            if let Some(s) = scope {
                debug!("Scope: {}", s);
            }
            if let Some(t) = task_type {
                debug!("Type: {}", t);
            }

            // Use TaskManager to add a new task
            if task_manager
                .add_task(description, *priority, scope.clone(), task_type.clone())
                .inspect(|id| display_manager.show_success(&format!("Added task with ID: {}", id)))
                .inspect_err(|e| display_manager.show_failure(&format!("Fail to ass task: {}", e)))
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        cli::commands::Commands::List {
            priority,
            scope,
            task_type,
            status,
            from_date,
            to_date,
            fuzzy,
            stats,
        } => {
            trace!("List tasks");
            // TODO: Refactor filter options into dedicated struct, and implement function
            // to log filter options if let Some(p) = priority {
            //     display_manager.show_debug("Filter by priority: {}", p);
            // }
            // if let Some(s) = scope {
            //     display_manager.show_debug("Filter by scope: {}", s);
            // }
            // if let Some(t) = task_type {
            //     display_manager.show_debug("Filter by type: {:?}", t);
            // }
            // if let Some(s) = status {
            //     display_manager.show_debug("Filter by status: {}", s);
            // }
            // if let Some(f) = from_date {
            //     display_manager.show_debug("Filter by completion date from: {}", f);
            // }
            // if let Some(t) = to_date {
            //     display_manager.show_debug("Filter by completion date to: {}", t);
            // }
            // if let Some(f) = fuzzy {
            //     display_manager.show_debug("Search using fuzzy match: {}", f);
            // }
            debug!("Show statistics: {}", stats);

            // Use TaskManager to list tasks
            let Ok(tasks) = task_manager
                .list_tasks(
                    *priority,
                    scope.as_deref(),
                    task_type.clone(),
                    *status,
                    from_date.as_deref(),
                    to_date.as_deref(),
                    fuzzy.as_deref(),
                )
                .inspect_err(|e| {
                    display_manager.show_failure(&format!("Fail to load tasks: {}", e));
                })
            else {
                return ExitCode::FAILURE;
            };

            // Check if tasks are empty
            if tasks.is_empty() {
                display_manager.show_success("No tasks found");
                return ExitCode::SUCCESS;
            }

            // Use DisplayManager to show tasks
            if let Err(e) = display_manager.show_tasks(&tasks, *stats) {
                display_manager.show_failure(&format!("Fail to show tasks: {}", e));
                return ExitCode::FAILURE;
            }
        }
        cli::commands::Commands::Done { id } => {
            trace!("Mark task {} as completed", id);

            // Use TaskManager to mark task as completed
            if task_manager
                .mark_task_done(id)
                .inspect(|_| display_manager.show_success(&format!("Task {} marked as done", id)))
                .inspect_err(|e| {
                    display_manager.show_failure(&format!("Fail to mark task as done: {}", e))
                })
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        cli::commands::Commands::Edit { id } => {
            trace!("Edit task {}", id);

            // Use TaskManager to edit task description
            if task_manager
                .edit_task_description(id)
                .inspect(|id| display_manager.show_success(&format!("Updated task {}", id)))
                .inspect_err(|e| {
                    display_manager.show_failure(&format!("Fail to update task: {}", e))
                })
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        cli::commands::Commands::Start { id } => {
            trace!("Start task {}", id);

            // Use TaskManager to start a task
            if task_manager
                .start_task(id)
                .inspect(|id| display_manager.show_success(&format!("Started task {}", id)))
                .inspect_err(|e| {
                    display_manager.show_failure(&format!("Fail to start task: {}", e))
                })
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        cli::commands::Commands::Stop {} => {
            trace!("Stop active task");

            // Use TaskManager to stop the active task
            if task_manager
                .stop_task()
                .inspect(|id| display_manager.show_success(&format!("Stopped task {}", id)))
                .inspect_err(|e| display_manager.show_failure(&format!("Fail to stop task: {}", e)))
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        cli::commands::Commands::Abort { id } => {
            if let Some(id) = id {
                trace!("Abort task {}", id);
            } else {
                trace!("Abort active task");
            }

            // Use TaskManager to abort a task
            if task_manager
                .abort_task(id)
                .inspect(|id| display_manager.show_success(&format!("Aborted task {}", id)))
                .inspect_err(|e| {
                    display_manager.show_failure(&format!("Fail to abort task: {}", e))
                })
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        cli::commands::Commands::Clean {
            priority,
            scope,
            task_type,
            status,
            older_than,
            force,
        } => {
            trace!("Clean tasks");
            // TODO: Use the same filter options as in the list command
            // if let Some(p) = priority {
            //     display_manager.show_debug("Filter by priority: {}", p);
            // }
            // if let Some(s) = scope {
            //     display_manager.show_debug("Filter by scope: {}", s);
            // }
            // if let Some(t) = task_type {
            //     display_manager.show_debug("Filter by type: {:?}", t);
            // }
            // if let Some(s) = status {
            //     display_manager.show_debug("Filter by status: {}", s);
            // }
            // if let Some(days) = older_than {
            //     display_manager.show_debug("Filter by age: older than {} days", days);
            // }
            debug!("Force clean without confirmation: {}", force);

            // Use TaskManager to clean tasks
            if task_manager
                .clean_tasks(
                    *priority,
                    scope.as_deref(),
                    task_type.clone(),
                    *status,
                    *older_than,
                    *force,
                    &display_manager,
                )
                .inspect(|count| display_manager.show_success(&format!("Cleaned {} tasks", count)))
                .inspect_err(|e| {
                    display_manager.show_failure(&format!("Fail to clean tasks: {}", e))
                })
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        cli::commands::Commands::Sync { prefer } => {
            trace!("Sync with remote repository");
            debug!("Conflict resolution preference: {}", prefer);

            // Use TaskManager to sync with remote repository
            if task_manager
                .sync(*prefer)
                .inspect(|_| {
                    display_manager.show_success("Successfully synced with remote repository")
                })
                .inspect_err(|e| {
                    display_manager.show_failure(&format!("Fail to sync tasks: {}", e))
                })
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        cli::commands::Commands::Clone { url } => {
            trace!("Clone remote repository");
            debug!("Repository URL: {}", url);

            // Use TaskManager to clone a remote repository
            if task_manager
                .clone_repo(url)
                .inspect(|_| {
                    display_manager
                        .show_success(&format!("Successfully cloned remote repository: {}", url))
                })
                .inspect_err(|e| {
                    display_manager.show_failure(&format!("Fail to clone repository: {}", e))
                })
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
    }

    // Catch-all for normal exit
    ExitCode::SUCCESS
}
