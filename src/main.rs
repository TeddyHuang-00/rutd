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

            // Use TaskManager to list tasks
            if let Ok(tasks) = task_manager
                .list_tasks(*priority, scope.as_deref(), task_type.clone(), *status)
                .inspect_err(|e| error!("Error loading task list: {}", e))
            {
                // Display the task list
                if tasks.is_empty() {
                    info!("No tasks found matching the criteria.");
                }

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
                .inspect_err(|e| error!("Fail to mark task as finished: {}", e))
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
                .inspect_err(|e| error!("Fail to update task: {}", e))
                .is_ok()
            {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
