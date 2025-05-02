use anyhow::Result;
use comfy_table::{Cell, CellAlignment, ContentArrangement, Row, Table};
use dialoguer::Confirm;

use crate::task::{Priority, Task, TaskStatus};

/// Display options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Command Line Interface mode
    Cli,
    /// Text User Interface mode
    Tui,
}

/// Responsible for handling all user interface output
#[derive(Debug)]
pub struct DisplayManager {
    /// Current display mode
    mode: DisplayMode,
}

impl Default for DisplayManager {
    fn default() -> Self {
        Self::new(DisplayMode::Cli)
    }
}

impl DisplayManager {
    /// Create a new DisplayManager
    pub fn new(mode: DisplayMode) -> Self {
        Self { mode }
    }

    /// Get the current display mode
    pub fn mode(&self) -> DisplayMode {
        self.mode
    }

    /// Set the display mode
    pub fn set_mode(&mut self, mode: DisplayMode) {
        self.mode = mode;
    }

    /// Display a success message
    pub fn show_success(&self, message: &str) {
        match self.mode {
            // TODO: Replace with a proper success message, instead of logging
            DisplayMode::Cli => log::info!("{}", message),
            // TODO: Implement TUI mode
            DisplayMode::Tui => unimplemented!(),
        }
    }

    /// Display a failure message
    pub fn show_failure(&self, message: &str) {
        match self.mode {
            // TODO: Replace with a proper failure message, instead of logging
            DisplayMode::Cli => log::error!("{}", message),
            // TODO: Implement TUI mode
            DisplayMode::Tui => unimplemented!(),
        }
    }

    /// Display the task list
    pub fn show_tasks(&self, tasks: &[Task], show_stats: bool) -> Result<()> {
        match self.mode {
            DisplayMode::Cli => self.show_tasks_cli(tasks, show_stats),
            // TODO: Implement TUI mode
            DisplayMode::Tui => unimplemented!(),
        }
    }

    /// Display the task list in CLI mode
    fn show_tasks_cli(&self, tasks: &[Task], show_stats: bool) -> Result<()> {
        if tasks.is_empty() {
            return Ok(());
        }

        // Create a table
        let mut table = Table::new();
        table
            .set_header(vec![
                "ID",
                "Description",
                "Priority",
                "Status",
                "Scope",
                "Type",
                "Time Spent",
                "Completed At",
            ])
            .set_content_arrangement(ContentArrangement::Dynamic)
            .load_preset(comfy_table::presets::UTF8_FULL)
            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);

        // Add rows to the table
        for task in tasks {
            let mut row = Row::new();

            // Use the first 8 characters of the task ID, should be unique
            row.add_cell(Cell::new(&task.id[0..8]));

            // Description
            row.add_cell(Cell::new(&task.description));

            // Priority with proper color
            row.add_cell(self.format_priority_cell(&task.priority));

            // Status with proper color
            row.add_cell(self.format_status_cell(&task.status));

            // Scope
            row.add_cell(
                Cell::new(task.scope.as_deref().unwrap_or("-"))
                    .set_alignment(CellAlignment::Center),
            );

            // Type
            row.add_cell(
                Cell::new(task.task_type.as_deref().unwrap_or("-"))
                    .set_alignment(CellAlignment::Center),
            );

            // Time spent
            let time_spent = if let Some(ts) = task.time_spent {
                let hours = ts / 3600;
                let minutes = (ts % 3600) / 60;
                let seconds = ts % 60;
                format!("{}h {}m {}s", hours, minutes, seconds)
            } else {
                "-".to_string()
            };
            row.add_cell(Cell::new(time_spent).set_alignment(CellAlignment::Right));

            // Completed at
            row.add_cell(Cell::new(task.completed_at.as_deref().unwrap_or("-")));

            table.add_row(row);
        }

        // Finalize the table and print it
        println!("{table}");

        // Show statistics if requested
        if show_stats {
            self.show_task_stats(tasks)?;
        }

        Ok(())
    }

    /// Format priority cell (with color)
    fn format_priority_cell(&self, priority: &Priority) -> Cell {
        let cell = Cell::new(priority.to_string());
        match priority {
            Priority::Urgent => cell.fg(comfy_table::Color::Red),
            Priority::High => cell.fg(comfy_table::Color::Yellow),
            Priority::Normal => cell.fg(comfy_table::Color::Green),
            Priority::Low => cell.fg(comfy_table::Color::Blue),
        }
    }

    /// Format status cell (with color)
    fn format_status_cell(&self, status: &TaskStatus) -> Cell {
        let cell = Cell::new(status.to_string());
        match status {
            TaskStatus::Todo => cell.fg(comfy_table::Color::Blue),
            TaskStatus::Done => cell.fg(comfy_table::Color::Green),
            TaskStatus::Aborted => cell.fg(comfy_table::Color::Red),
        }
    }

    /// Display task statistics
    fn show_task_stats(&self, tasks: &[Task]) -> Result<()> {
        let mut stats_table = Table::new();
        stats_table
            .load_preset(comfy_table::presets::UTF8_FULL)
            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);

        // Total tasks
        stats_table.add_row(vec!["Total tasks", &tasks.len().to_string()]);

        // Task counts by status
        let todo_count = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Todo)
            .count();
        let done_count = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .count();
        let aborted_count = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Aborted)
            .count();

        stats_table.add_row(vec!["Pending tasks", &todo_count.to_string()]);
        stats_table.add_row(vec!["Finished tasks", &done_count.to_string()]);
        stats_table.add_row(vec!["Cancelled tasks", &aborted_count.to_string()]);

        // Total time spent
        let total_time_spent: u64 = tasks.iter().filter_map(|t| t.time_spent).sum();
        let hours = total_time_spent / 3600;
        let minutes = (total_time_spent % 3600) / 60;
        let seconds = total_time_spent % 60;
        stats_table.add_row(vec![
            "Total time spent",
            &format!("{}h {}m {}s", hours, minutes, seconds),
        ]);

        println!("\n{stats_table}");
        Ok(())
    }

    /// Get user confirmation
    pub fn confirm(&self, message: &str) -> Result<bool> {
        match self.mode {
            DisplayMode::Cli => {
                let confirmed = Confirm::new().with_prompt(message).interact()?;
                Ok(confirmed)
            }
            // TODO: Implement TUI mode
            DisplayMode::Tui => unimplemented!(),
        }
    }

    /// Display details for a specific task
    pub fn show_task_detail(&self, task: &Task) -> Result<()> {
        match self.mode {
            DisplayMode::Cli => {
                let mut table = Table::new();
                table
                    .load_preset(comfy_table::presets::UTF8_FULL)
                    .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);

                table.add_row(vec!["ID", &task.id]);
                table.add_row(vec!["Description", &task.description]);
                table.add_row(vec!["Priority", &task.priority.to_string()]);
                table.add_row(vec!["Status", &task.status.to_string()]);

                if let Some(scope) = &task.scope {
                    table.add_row(vec!["Scope", scope]);
                }

                if let Some(task_type) = &task.task_type {
                    table.add_row(vec!["Type", task_type]);
                }

                table.add_row(vec!["Created at", &task.created_at]);

                if let Some(updated_at) = &task.updated_at {
                    table.add_row(vec!["Updated at", updated_at]);
                }

                if let Some(completed_at) = &task.completed_at {
                    table.add_row(vec!["Completed at", completed_at]);
                }

                if let Some(time_spent) = task.time_spent {
                    let hours = time_spent / 3600;
                    let minutes = (time_spent % 3600) / 60;
                    let seconds = time_spent % 60;
                    table.add_row(vec![
                        "Time spent",
                        &format!("{}h {}m {}s", hours, minutes, seconds),
                    ]);
                }

                println!("{table}");
                Ok(())
            }
            // TODO: Implement TUI mode
            DisplayMode::Tui => unimplemented!(),
        }
    }
}
