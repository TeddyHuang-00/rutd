use anyhow::Result;

use crate::task::Task;

/// Display interface for core functionality
///
/// This trait is responsible for handling all user interface output
/// and is implemented for different display modes (CLI, TUI, etc.)
pub trait Display {
    /// Get user confirmation
    fn confirm(&self, message: &str) -> Result<bool>;
    /// Editor for user input
    fn edit(&self, message: &str) -> Result<String>;
    /// Display a success message
    fn show_success(&self, message: &str);
    /// Display a failure message
    fn show_failure(&self, message: &str);
    /// Display the task list
    fn show_tasks_list(&self, tasks: &[Task]);
    /// Display task statistics
    fn show_task_stats(&self, tasks: &[Task]);
    /// Display details for a specific task
    fn show_task_detail(&self, task: &Task);
}
