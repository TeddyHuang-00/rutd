pub mod cli;

use std::process::ExitCode;

use clap::Parser;
use cli::{Cli, Commands, DisplayManager};
use rutd_core::{Config, Display, TaskManager};

pub fn app() -> ExitCode {
    // Parse command line arguments
    let cli = Cli::parse();

    // Get configuration from environment variables
    let Ok(config) = Config::new().inspect_err(|e| eprintln!("Failed to load configuration: {e}"))
    else {
        return ExitCode::FAILURE;
    };

    // Initialize logging system
    if let Err(e) = rutd_core::logging::init_logger(
        cli.verbose,
        config.path.log_file_path(),
        config.log.history,
        config.log.console,
    ) {
        eprintln!("{e}");
        return ExitCode::FAILURE;
    }

    log::trace!("Received cli args: {cli:?}");
    log::trace!("Loaded configuration: {config:?}");

    let path_config = config.path;
    let git_config = config.git;

    // Create a display manager
    let display_manager = DisplayManager;

    // Build the task manager
    let task_manager = TaskManager::new(path_config, git_config);

    // Handle different commands
    match cli.command {
        Commands::Add {
            description,
            priority,
            task_scope: scope,
            task_type,
        } => {
            let priority = priority.into();
            log::trace!("Add task command");
            log::debug!("Add task: {description}");
            log::debug!("Priority: {priority}");
            let scope = scope.inspect(|s| log::debug!("Task scope: {s}"));
            let task_type = task_type.inspect(|t| log::debug!("Task type: {t}"));

            // Use TaskManager to add a new task
            if task_manager
                .add_task(&description, priority, scope, task_type)
                .inspect(|id| display_manager.show_success(&format!("Added task with ID: {id}")))
                .inspect_err(|e| display_manager.show_failure(&format!("Fail to add task: {e}")))
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        Commands::List { filter, stats } => {
            log::trace!("List tasks");
            // Use the FilterOptions struct instead of individual parameters

            // Use TaskManager to list tasks
            let Ok(tasks) = task_manager.list_tasks(&filter.into()).inspect_err(|e| {
                display_manager.show_failure(&format!("Fail to load tasks: {e}"));
            }) else {
                return ExitCode::FAILURE;
            };

            // Check if tasks are empty
            if tasks.is_empty() {
                display_manager.show_success("No tasks found");
                return ExitCode::SUCCESS;
            }

            // Use DisplayManager to show tasks
            display_manager.show_tasks_list(&tasks);

            if stats {
                display_manager.show_task_stats(&tasks);
            }
        }
        Commands::Done { id } => {
            log::trace!("Mark task {id} as completed");

            // Use TaskManager to mark task as completed
            if task_manager
                .mark_task_done(&id)
                .inspect(|_| display_manager.show_success(&format!("Task {id} marked as done")))
                .inspect_err(|e| {
                    display_manager.show_failure(&format!("Fail to mark task as done: {e}"))
                })
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        Commands::Edit { id } => {
            log::trace!("Edit task {id}");

            // Use TaskManager to edit task description
            if task_manager
                .edit_task_description(&id, &display_manager)
                .inspect(|id| display_manager.show_success(&format!("Updated task {id}")))
                .inspect_err(|e| display_manager.show_failure(&format!("Fail to update task: {e}")))
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        Commands::Start { id } => {
            log::trace!("Start task {id}");

            // Use TaskManager to start a task
            if task_manager
                .start_task(&id)
                .inspect(|id| display_manager.show_success(&format!("Started task {id}")))
                .inspect_err(|e| display_manager.show_failure(&format!("Fail to start task: {e}")))
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        Commands::Stop {} => {
            log::trace!("Stop active task");

            // Use TaskManager to stop the active task
            if task_manager
                .stop_task()
                .inspect(|id| display_manager.show_success(&format!("Stopped task {id}")))
                .inspect_err(|e| display_manager.show_failure(&format!("Fail to stop task: {e}")))
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        Commands::Abort { id } => {
            let id = id
                .inspect(|id| {
                    log::trace!("Abort task {id}");
                })
                .or_else(|| {
                    log::trace!("Abort active task");
                    None
                });

            // Use TaskManager to abort a task
            if task_manager
                .abort_task(&id)
                .inspect(|id| display_manager.show_success(&format!("Aborted task {id}")))
                .inspect_err(|e| display_manager.show_failure(&format!("Fail to abort task: {e}")))
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        Commands::Clean { filter, force } => {
            log::trace!("Clean tasks");
            // Use the FilterOptions struct instead of individual parameters
            log::debug!("Force clean without confirmation: {force}");

            // Use TaskManager to clean tasks
            if task_manager
                .clean_tasks(&filter.into(), force, &display_manager)
                .inspect(|count| display_manager.show_success(&format!("Cleaned {count} tasks")))
                .inspect_err(|e| display_manager.show_failure(&format!("Fail to clean tasks: {e}")))
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        Commands::Sync { prefer } => {
            let prefer = prefer.into();
            log::trace!("Sync with remote repository");
            log::debug!("Conflict resolution preference: {prefer}");

            // Use TaskManager to sync with remote repository
            if task_manager
                .sync(prefer)
                .inspect(|_| {
                    display_manager.show_success("Successfully synced with remote repository")
                })
                .inspect_err(|e| display_manager.show_failure(&format!("Fail to sync tasks: {e}")))
                .is_err()
            {
                return ExitCode::FAILURE;
            }
        }
        Commands::Clone { url } => {
            log::trace!("Clone remote repository");
            log::debug!("Repository URL: {url}");

            // Use TaskManager to clone a remote repository
            if task_manager
                .clone_repo(&url)
                .inspect(|_| {
                    display_manager
                        .show_success(&format!("Successfully cloned remote repository: {url}"))
                })
                .inspect_err(|e| {
                    display_manager.show_failure(&format!("Fail to clone repository: {e}"))
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
