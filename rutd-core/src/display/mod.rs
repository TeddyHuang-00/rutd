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
    ///
    /// Should return `None` if the user aborts the action or no changes are
    /// made, and `Some(String)` if the user provides a new value.
    fn edit(&self, message: &str) -> Result<Option<String>>;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{Priority, Task};

    // A simple mock implementation of the Display trait for testing
    struct MockDisplay {
        confirm_result: bool,
        edit_result: Option<String>,
        should_fail: bool,
    }

    impl MockDisplay {
        fn new(confirm_result: bool, edit_result: Option<String>, should_fail: bool) -> Self {
            Self {
                confirm_result,
                edit_result,
                should_fail,
            }
        }
    }

    impl Display for MockDisplay {
        fn confirm(&self, _message: &str) -> Result<bool> {
            if self.should_fail {
                anyhow::bail!("Confirmation failed")
            } else {
                Ok(self.confirm_result)
            }
        }

        fn edit(&self, _message: &str) -> Result<Option<String>> {
            if self.should_fail {
                anyhow::bail!("Edit failed")
            } else {
                Ok(self.edit_result.clone())
            }
        }

        // Do nothing in the mock implementation
        fn show_success(&self, _message: &str) {}

        // Do nothing in the mock implementation
        fn show_failure(&self, _message: &str) {}

        // Do nothing in the mock implementation
        fn show_tasks_list(&self, _tasks: &[Task]) {}

        // Do nothing in the mock implementation
        fn show_task_stats(&self, _tasks: &[Task]) {}

        // Do nothing in the mock implementation
        fn show_task_detail(&self, _task: &Task) {}
    }

    #[test]
    fn test_display_confirm() {
        // Create a mock display that returns true for confirmation
        let display = MockDisplay::new(true, None, false);

        // Test confirmation
        let result = display.confirm("Are you sure?");
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Create a mock display that returns false for confirmation
        let display = MockDisplay::new(false, None, false);

        // Test confirmation
        let result = display.confirm("Are you sure?");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Create a mock display that fails
        let display = MockDisplay::new(false, None, true);

        // Test failing confirmation
        let result = display.confirm("Are you sure?");
        assert!(result.is_err());
    }

    #[test]
    fn test_display_edit() {
        // Create a mock display that returns Some for edit
        let edit_text = "Edited text".to_string();
        let display = MockDisplay::new(false, Some(edit_text.clone()), false);

        // Test edit
        let result = display.edit("Edit this text");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(edit_text));

        // Create a mock display that returns None for edit (user aborted)
        let display = MockDisplay::new(false, None, false);

        // Test edit with abort
        let result = display.edit("Edit this text");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);

        // Create a mock display that fails
        let display = MockDisplay::new(false, None, true);

        // Test failing edit
        let result = display.edit("Edit this text");
        assert!(result.is_err());
    }

    #[test]
    fn test_display_show_methods() {
        // These methods don't return anything, so we just verify they don't panic
        let display = MockDisplay::new(false, None, false);

        // Create a sample task
        let task = Task::new(
            "test-id".to_string(),
            "Test task".to_string(),
            Priority::Normal,
            None,
            None,
        );

        // Test all show methods
        display.show_success("Success message");
        display.show_failure("Failure message");
        display.show_tasks_list(&[task.clone()]);
        display.show_task_stats(&[task.clone()]);
        display.show_task_detail(&task);
    }
}
