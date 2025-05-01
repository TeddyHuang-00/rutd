mod cli;
mod task;
mod git;

use cli::Cli;
use clap::Parser;

fn main() {
    let cli = Cli::parse();

    // Handle different commands
    match &cli.command {
        cli::commands::Commands::Add { description, priority, scope, task_type } => {
            println!("Add task: {}", description);
            println!("Priority: {}", priority);
            if let Some(s) = scope {
                println!("Scope: {}", s);
            }
            println!("Type: {:?}", task_type);
            // TODO: Add task logic
        }
        cli::commands::Commands::List { priority, scope, task_type, status } => {
            println!("List tasks");
            if let Some(p) = priority {
                println!("Filter by priority: {}", p);
            }
            if let Some(s) = scope {
                println!("Filter by scope: {}", s);
            }
            if let Some(t) = task_type {
                println!("Filter by type: {:?}", t);
            }
            if let Some(s) = status {
                println!("Filter by status: {}", s);
            }
            // TODO: Add logic to list tasks
        }
        cli::commands::Commands::Done { id } => {
            println!("Mark task {} as completed", id);
            // TODO: Add logic to mark task as done
        }
        cli::commands::Commands::Edit { id } => {
            println!("Edit task {}", id);
            // TODO: Add logic to edit task
        }
    }
}
