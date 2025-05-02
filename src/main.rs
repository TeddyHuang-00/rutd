mod cli;
mod git;
mod task;

use std::{env, process::ExitCode};

use clap::Parser;
use cli::Cli;
use log::{LevelFilter, debug, error, info, trace, warn};
use simple_logger::SimpleLogger;
use task::TaskManager;

fn main() -> ExitCode {
    let cli = Cli::parse();
    // Set up logging
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level(
            env!("CARGO_PKG_NAME"),
            match cli.verbose {
                0 => LevelFilter::Info,
                1 => LevelFilter::Debug,
                2.. => LevelFilter::Trace,
            },
        )
        .without_timestamps()
        .init()
        .unwrap();

    trace!("Received cli args: {:?}", cli);

    // Build the task manager
    let task_manager = env::var("RUTD_TASKS_DIR")
        .map(|dir| TaskManager::new(&dir))
        .unwrap_or_default();

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
                .inspect(|id| info!("Added task ID: {}", id))
                .inspect_err(|e| error!("Failed to save task: {}", e))
                .is_ok()
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
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
            if let Some(p) = priority {
                debug!("Filter by priority: {}", p);
            }
            if let Some(s) = scope {
                debug!("Filter by scope: {}", s);
            }
            if let Some(t) = task_type {
                debug!("Filter by type: {:?}", t);
            }
            if let Some(s) = status {
                debug!("Filter by status: {}", s);
            }
            if let Some(f) = from_date {
                debug!("Filter by completion date from: {}", f);
            }
            if let Some(t) = to_date {
                debug!("Filter by completion date to: {}", t);
            }
            if let Some(f) = fuzzy {
                debug!("Search using fuzzy match: {}", f);
            }
            debug!("Show statistics: {}", stats);

            // Use TaskManager to list tasks
            if let Ok(tasks) = task_manager
                .list_tasks(
                    *priority,
                    scope.as_deref(),
                    task_type.clone(),
                    *status,
                    from_date.as_deref(),
                    to_date.as_deref(),
                    fuzzy.as_deref(),
                    *stats,
                )
                .inspect_err(|e| error!("Error loading task list: {}", e))
            {
                // Display the task list if tasks are found
                if tasks.is_empty() {
                    info!("No tasks found matching the criteria.");
                } else {
                    // TODO: Display tasks in a more user-friendly format such as in tables
                    for task in tasks {
                        info!("- ID: {}", task.id);
                        info!("  Description: {}", task.description);
                        info!("  Priority: {}", task.priority);
                        info!("  Status: {}", task.status);
                        if let Some(sc) = &task.scope {
                            info!("  Scope: {}", sc);
                        }
                        if let Some(tt) = &task.task_type {
                            info!("  Type: {}", tt);
                        }
                        if let Some(ts) = task.time_spent {
                            let hours = ts / 3600;
                            let minutes = (ts % 3600) / 60;
                            let seconds = ts % 60;
                            info!("  Time spent: {}h {}m {}s", hours, minutes, seconds);
                        }
                        if let Some(completed_at) = &task.completed_at {
                            info!("  Completed at: {}", completed_at);
                        }
                    }
                }
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        cli::commands::Commands::Done { id } => {
            trace!("Mark task {} as completed", id);

            // Use TaskManager to mark task as done
            if task_manager
                .mark_task_done(id)
                .inspect(|_| info!("Finished task {}", id))
                .inspect_err(|e| error!("Failed to mark task as finished: {}", e))
                .is_ok()
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        cli::commands::Commands::Edit { id } => {
            trace!("Edit task {}", id);

            // Use TaskManager to edit task description
            if task_manager
                .edit_task_description(id)
                .inspect(|_| info!("Updated task {}", id))
                .inspect_err(|e| error!("Failed to update task: {}", e))
                .is_ok()
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        cli::commands::Commands::Start { id } => {
            trace!("Start task {}", id);

            // Use TaskManager to start task
            if task_manager
                .start_task(id)
                .inspect(|_| info!("Started task {}", id))
                .inspect_err(|e| error!("Failed to start task: {}", e))
                .is_ok()
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        cli::commands::Commands::Stop { id } => {
            trace!("Stop task {}", id);

            // Use TaskManager to stop task
            if task_manager
                .stop_task(id)
                .inspect(|_| info!("Stopped task {}", id))
                .inspect_err(|e| error!("Failed to stop task: {}", e))
                .is_ok()
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        cli::commands::Commands::Abort { id } => {
            trace!("Abort task {}", id);

            // Use TaskManager to abort task
            if task_manager
                .abort_task(id)
                .inspect(|_| info!("Aborted task {}", id))
                .inspect_err(|e| error!("Failed to abort task: {}", e))
                .is_ok()
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
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
            if let Some(p) = priority {
                debug!("Filter by priority: {}", p);
            }
            if let Some(s) = scope {
                debug!("Filter by scope: {}", s);
            }
            if let Some(t) = task_type {
                debug!("Filter by type: {:?}", t);
            }
            if let Some(s) = status {
                debug!("Filter by status: {}", s);
            }
            if let Some(days) = older_than {
                debug!("Filter by age: older than {} days", days);
            }
            debug!("Force clean without confirmation: {}", force);

            // Use TaskManager to clean tasks
            match task_manager.clean_tasks(
                *priority,
                scope.as_deref(),
                task_type.clone(),
                *status,
                *older_than,
                *force,
            ) {
                Ok(count) => {
                    info!("Removed {} tasks", count);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    error!("Failed to clean tasks: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        cli::commands::Commands::Sync { prefer } => {
            trace!("Sync with remote repository");
            debug!("Conflict resolution preference: {}", prefer);

            // Use TaskManager to sync with remote repository
            if task_manager
                .sync(*prefer)
                .inspect(|_| info!("Successfully synced tasks with remote repository"))
                .inspect_err(|e| error!("Failed to sync tasks: {}", e))
                .is_ok()
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        cli::commands::Commands::Clone { url } => {
            trace!("Clone remote repository");
            debug!("Repository URL: {}", url);

            // Use TaskManager to clone remote repository
            if task_manager
                .clone_repo(url)
                .inspect(|_| info!("Successfully cloned remote repository to tasks directory"))
                .inspect_err(|e| error!("Failed to clone repository: {}", e))
                .is_ok()
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
