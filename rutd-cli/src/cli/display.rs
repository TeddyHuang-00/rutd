use std::collections::BTreeMap;

use anyhow::Result;
use colored::Colorize;
use comfy_table::{Cell, CellAlignment, ContentArrangement, Row, Table};
use dialoguer::{Confirm, Editor};
use rutd_core::{
    display::Display,
    task::{Priority, Task, TaskStatus},
};

/// Responsible for handling all user interface output
#[derive(Debug, Default)]
pub struct DisplayManager;

impl DisplayManager {
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
}

impl Display for DisplayManager {
    fn confirm(&self, message: &str) -> Result<bool> {
        let confirmed = Confirm::new().with_prompt(message).interact()?;
        Ok(confirmed)
    }
    fn edit(&self, message: &str) -> Result<Option<String>> {
        // FIXME: Handle cases when EDITOR is not set
        Ok(Editor::new().edit(message)?)
    }
    fn show_success(&self, message: &str) {
        println!("{} {}", "✓".green().bold(), message.green());
    }
    fn show_failure(&self, message: &str) {
        eprintln!("{} {}", "✗".red().bold(), message.red());
    }
    /// Display the task list table
    fn show_tasks_list(&self, tasks: &[Task]) {
        if tasks.is_empty() {
            return;
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
            let time_spent = task.time_spent.map_or("-".to_string(), |ts| {
                let hours = ts / 3600;
                let minutes = (ts % 3600) / 60;
                let seconds = ts % 60;
                format!("{hours}h {minutes}m {seconds}s")
            });
            row.add_cell(Cell::new(time_spent).set_alignment(CellAlignment::Right));

            // Completed at
            let complete_at = task.completed_at.as_ref().map_or("-".to_string(), |t| {
                let date = t.parse::<chrono::DateTime<chrono::Utc>>().unwrap();
                date.format("%Y-%m-%d %H:%M:%S").to_string()
            });
            row.add_cell(Cell::new(complete_at));

            table.add_row(row);
        }

        // Finalize the table and print it
        println!("{table}");
    }
    /// Display task statistics
    fn show_task_stats(&self, tasks: &[Task]) {
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
            &format!("{hours}h {minutes}m {seconds}s"),
        ]);

        println!("\n{stats_table}");
    }

    /// Display details for a specific task
    fn show_task_detail(&self, task: &Task) {
        let mut table = Table::new();
        table
            .load_preset(comfy_table::presets::UTF8_FULL)
            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);

        table.add_row(vec!["ID", &task.id]);
        table.add_row(vec!["Description", &task.description]);
        table.add_row(vec!["Priority", task.priority.as_ref()]);
        table.add_row(vec!["Status", task.status.as_ref()]);

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
                &format!("{hours}h {minutes}m {seconds}s"),
            ]);
        }

        println!("{table}");
    }
}

impl DisplayManager {
    /// Display configuration values
    pub fn show_config_values(&self, values: &BTreeMap<String, String>) {
        let mut table = Table::new();
        table
            .load_preset(comfy_table::presets::UTF8_FULL)
            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS);

        table.set_header(vec!["Configuration Key", "Value"]);
        for (key, value) in values {
            table.add_row(vec![key, value]);
        }

        println!("{table}");
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;

    use super::*;

    fn create_test_task(id: &str, status: TaskStatus) -> Task {
        Task {
            id: id.to_string(),
            description: format!("Test task {id}"),
            priority: Priority::Normal,
            scope: Some("test-scope".to_string()),
            task_type: Some("test-type".to_string()),
            status,
            created_at: Local::now().to_rfc3339(),
            updated_at: Some(Local::now().to_rfc3339()),
            completed_at: if status == TaskStatus::Done || status == TaskStatus::Aborted {
                Some(Local::now().to_rfc3339())
            } else {
                None
            },
            time_spent: Some(3600), // 1 hour in seconds
        }
    }

    #[test]
    fn test_format_priority_cell() {
        let display = DisplayManager;

        // Test formatting for each priority level
        let urgent_cell = display.format_priority_cell(&Priority::Urgent);
        let high_cell = display.format_priority_cell(&Priority::High);
        let normal_cell = display.format_priority_cell(&Priority::Normal);
        let low_cell = display.format_priority_cell(&Priority::Low);

        // Check cell content (colors can't be easily tested)
        assert_eq!(urgent_cell.content(), "urgent");
        assert_eq!(high_cell.content(), "high");
        assert_eq!(normal_cell.content(), "normal");
        assert_eq!(low_cell.content(), "low");
    }

    #[test]
    fn test_format_status_cell() {
        let display = DisplayManager;

        // Test formatting for each status
        let todo_cell = display.format_status_cell(&TaskStatus::Todo);
        let done_cell = display.format_status_cell(&TaskStatus::Done);
        let aborted_cell = display.format_status_cell(&TaskStatus::Aborted);

        // Check cell content (colors can't be easily tested)
        assert_eq!(todo_cell.content(), "todo");
        assert_eq!(done_cell.content(), "done");
        assert_eq!(aborted_cell.content(), "aborted");
    }

    #[test]
    fn test_show_task_stats() {
        let display = DisplayManager;

        // Create tasks with different statuses
        let tasks = vec![
            create_test_task("task1", TaskStatus::Todo),
            create_test_task("task2", TaskStatus::Done),
            create_test_task("task3", TaskStatus::Aborted),
            create_test_task("task4", TaskStatus::Todo),
        ];

        // This is a visual test, so we just ensure it doesn't panic
        display.show_task_stats(&tasks);
        // For empty tasks
        display.show_task_stats(&[]);
    }

    #[test]
    fn test_show_tasks_list() {
        let display = DisplayManager;

        // Create a list of tasks
        let tasks = vec![
            create_test_task("task-123", TaskStatus::Todo),
            create_test_task("task-456", TaskStatus::Done),
            create_test_task("task-789", TaskStatus::Aborted),
        ];

        // This is a visual test, so we just ensure it doesn't panic
        display.show_tasks_list(&tasks);
        // For empty tasks
        display.show_tasks_list(&[]);
    }

    #[test]
    fn test_show_task_detail() {
        let display = DisplayManager;

        // Create a task with all details
        let task = create_test_task("task-detail", TaskStatus::Done);

        // This is a visual test, so we just ensure it doesn't panic
        display.show_task_detail(&task);

        // Test with minimal details
        let minimal_task = Task {
            id: "minimal".to_string(),
            description: "Minimal task".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Todo,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        };

        display.show_task_detail(&minimal_task);
    }
}
