mod cli;
mod git;
mod task;

use std::{
    env,
    fs::remove_file,
    process::{Command, ExitCode},
};

use chrono::Utc;
use clap::Parser;
use cli::Cli;
use log::{LevelFilter, debug, error, info, trace, warn};
use simple_logger::SimpleLogger;
use task::{
    model::{Priority, Task, TaskStatus},
    storage::{load_all_tasks, load_task, save_task},
};
use uuid::Uuid;

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

            // Create a new task
            let task_id = Uuid::new_v4().to_string();
            let task = Task::new(
                task_id.clone(),
                description.clone(),
                *priority,
                scope.clone(),
                task_type.clone(),
            );

            // Save the task
            if save_task(&task)
                .inspect(|_| info!("Added task ID: {}", task_id))
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

            // Load all tasks
            trace!("Loading all tasks");
            let Ok(mut tasks) =
                load_all_tasks().inspect_err(|e| error!("Error loading task list: {}", e))
            else {
                return ExitCode::FAILURE;
            };

            // Apply filters
            if let Some(p) = priority {
                tasks.retain(|t| &t.priority == p);
            }
            if let Some(s) = scope {
                tasks.retain(|t| t.scope.as_ref() == Some(s));
            }
            if let Some(tt) = task_type {
                tasks.retain(|t| t.task_type.as_ref() == Some(tt));
            }
            if let Some(s) = status {
                tasks.retain(|t| &t.status == s);
            }

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
        }
        cli::commands::Commands::Done { id } => {
            trace!("Mark task {} as completed", id);

            // Load task
            let Ok(mut task) =
                load_task(id).inspect_err(|e| error!("Failed to load todo list: {}", e))
            else {
                return ExitCode::FAILURE;
            };
            // Update task status to Done
            task.status = TaskStatus::Done;
            task.completed_at = Some(Utc::now().to_rfc3339());
            task.updated_at = Some(Utc::now().to_rfc3339());

            // Save the updated task
            if save_task(&task)
                .inspect(|_| info!("Finished task {}", task.id))
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

            // Load task
            let Ok(mut task) =
                load_task(id).inspect_err(|e| error!("Fail to load todo list: {}", e))
            else {
                return ExitCode::FAILURE;
            };
            // Get the editor from environment variable or default to "nano"
            let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

            // Create a temporary file for editing
            let temp_file = format!("/tmp/rutd_edit_{}.txt", task.id);
            std::fs::write(&temp_file, &task.description).expect("Unable to write temporary file");

            // Open the editor
            let status = Command::new(&editor)
                .arg(&temp_file)
                .status()
                .expect("Unable to open editor");

            if !status.success() {
                error!("Editing canceled or failed");
                return ExitCode::FAILURE;
            }

            // Read the updated content from the temporary file
            let new_description =
                std::fs::read_to_string(&temp_file).expect("Unable to read temporary file");
            task.description = new_description.trim().to_string();
            task.updated_at = Some(chrono::Utc::now().to_rfc3339());

            // Save the updated task
            let success = save_task(&task)
                .inspect(|_| info!("Updated task {}", task.id))
                .inspect_err(|e| error!("Fail to update task: {}", e))
                .is_ok();

            // Clean up the temporary file
            let success = remove_file(&temp_file)
                .inspect(|_| info!("Deleted temporary file {}", temp_file))
                .inspect_err(|e| error!("Failed to delete temporary file: {}", e))
                .is_ok()
                && success;
            if success {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}
