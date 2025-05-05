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
    use std::{cell::RefCell, collections::VecDeque};

    use super::*;
    use crate::task::Priority;

    /// A real test implementation of the Display trait
    /// This implementation captures all output for verification
    struct TestDisplay {
        // Store confirmation responses to return for successive calls
        confirm_responses: RefCell<VecDeque<bool>>,
        // Store edit responses to return for successive calls
        edit_responses: RefCell<VecDeque<Option<String>>>,
        // Capture success messages for verification
        success_messages: RefCell<Vec<String>>,
        // Capture failure messages for verification
        failure_messages: RefCell<Vec<String>>,
        // Track if task list was shown
        tasks_list_shown: RefCell<bool>,
        // Track if task stats were shown
        task_stats_shown: RefCell<bool>,
        // Track if task detail was shown and which task
        task_detail_shown: RefCell<Option<String>>,
        // Set to true to simulate failures
        should_fail: bool,
    }

    impl TestDisplay {
        fn new(should_fail: bool) -> Self {
            Self {
                confirm_responses: RefCell::new(VecDeque::new()),
                edit_responses: RefCell::new(VecDeque::new()),
                success_messages: RefCell::new(Vec::new()),
                failure_messages: RefCell::new(Vec::new()),
                tasks_list_shown: RefCell::new(false),
                task_stats_shown: RefCell::new(false),
                task_detail_shown: RefCell::new(None),
                should_fail,
            }
        }

        /// Queue a confirmation response to be returned on next call
        fn queue_confirm(&self, response: bool) {
            self.confirm_responses.borrow_mut().push_back(response);
        }

        /// Queue an edit response to be returned on next call
        fn queue_edit(&self, response: Option<String>) {
            self.edit_responses.borrow_mut().push_back(response);
        }

        /// Get all success messages that were shown
        fn get_success_messages(&self) -> Vec<String> {
            self.success_messages.borrow().clone()
        }

        /// Get all failure messages that were shown
        fn get_failure_messages(&self) -> Vec<String> {
            self.failure_messages.borrow().clone()
        }

        /// Check if task list was shown
        fn was_tasks_list_shown(&self) -> bool {
            *self.tasks_list_shown.borrow()
        }

        /// Check if task stats were shown
        fn were_task_stats_shown(&self) -> bool {
            *self.task_stats_shown.borrow()
        }

        /// Get the ID of the task whose details were shown, if any
        fn get_task_detail_shown(&self) -> Option<String> {
            self.task_detail_shown.borrow().clone()
        }
    }

    impl Display for TestDisplay {
        fn confirm(&self, message: &str) -> Result<bool> {
            if self.should_fail {
                anyhow::bail!("Confirmation failed for: {}", message)
            } else {
                match self.confirm_responses.borrow_mut().pop_front() {
                    Some(response) => Ok(response),
                    None => anyhow::bail!("No confirmation response queued for: {}", message),
                }
            }
        }

        fn edit(&self, message: &str) -> Result<Option<String>> {
            if self.should_fail {
                anyhow::bail!("Edit failed for: {}", message)
            } else {
                match self.edit_responses.borrow_mut().pop_front() {
                    Some(response) => Ok(response),
                    None => anyhow::bail!("No edit response queued for: {}", message),
                }
            }
        }

        fn show_success(&self, message: &str) {
            self.success_messages.borrow_mut().push(message.to_string());
        }

        fn show_failure(&self, message: &str) {
            self.failure_messages.borrow_mut().push(message.to_string());
        }

        fn show_tasks_list(&self, _tasks: &[Task]) {
            *self.tasks_list_shown.borrow_mut() = true;
        }

        fn show_task_stats(&self, _tasks: &[Task]) {
            *self.task_stats_shown.borrow_mut() = true;
        }

        fn show_task_detail(&self, task: &Task) {
            *self.task_detail_shown.borrow_mut() = Some(task.id.clone());
        }
    }

    #[test]
    fn test_display_confirm() {
        // Create a test display
        let display = TestDisplay::new(false);

        // Queue a positive confirmation response
        display.queue_confirm(true);

        // Test confirmation with the queued response
        let result = display.confirm("Are you sure?");
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Queue a negative confirmation response
        display.queue_confirm(false);

        // Test confirmation with the queued response
        let result = display.confirm("Are you sure?");
        assert!(result.is_ok());
        assert!(!result.unwrap());

        // Create a failing display
        let display = TestDisplay::new(true);

        // Test failing confirmation
        let result = display.confirm("Are you sure?");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Are you sure?"));
    }

    #[test]
    fn test_display_edit() {
        // Create a test display
        let display = TestDisplay::new(false);

        // Queue an edit response with text
        let edit_text = "Edited text".to_string();
        display.queue_edit(Some(edit_text.clone()));

        // Test edit with the queued response
        let result = display.edit("Edit this text");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some(edit_text));

        // Queue a None edit response (user aborted)
        display.queue_edit(None);

        // Test edit with the queued abort response
        let result = display.edit("Edit this text");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);

        // Create a failing display
        let display = TestDisplay::new(true);

        // Test failing edit
        let result = display.edit("Edit this text");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Edit this text"));
    }

    #[test]
    fn test_display_show_methods() {
        // Create a test display
        let display = TestDisplay::new(false);

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

        // Verify the success message was captured
        let success_messages = display.get_success_messages();
        assert_eq!(success_messages.len(), 1);
        assert_eq!(success_messages[0], "Success message");

        // Verify the failure message was captured
        let failure_messages = display.get_failure_messages();
        assert_eq!(failure_messages.len(), 1);
        assert_eq!(failure_messages[0], "Failure message");

        // Verify task list was shown
        assert!(display.was_tasks_list_shown());

        // Verify task stats were shown
        assert!(display.were_task_stats_shown());

        // Verify task detail was shown
        let task_detail = display.get_task_detail_shown();
        assert!(task_detail.is_some());
        assert_eq!(task_detail.unwrap(), "test-id");
    }

    #[test]
    fn test_display_multiple_operations() {
        // Create a test display
        let display = TestDisplay::new(false);

        // Queue multiple responses
        display.queue_confirm(true);
        display.queue_edit(Some("First edit".to_string()));
        display.queue_confirm(false);
        display.queue_edit(None);

        // Create sample tasks
        let task1 = Task::new(
            "task-1".to_string(),
            "Task 1".to_string(),
            Priority::Normal,
            None,
            None,
        );
        let task2 = Task::new(
            "task-2".to_string(),
            "Task 2".to_string(),
            Priority::High,
            None,
            None,
        );

        // Execute a sequence of operations
        let _ = display.confirm("First confirmation");
        let _ = display.edit("First edit");
        display.show_success("First success");
        display.show_tasks_list(&[task1.clone(), task2.clone()]);
        let _ = display.confirm("Second confirmation");
        let _ = display.edit("Second edit");
        display.show_failure("First failure");
        display.show_task_detail(&task2);

        // Verify all operations were recorded correctly
        assert_eq!(display.get_success_messages().len(), 1);
        assert_eq!(display.get_failure_messages().len(), 1);
        assert!(display.was_tasks_list_shown());
        assert_eq!(display.get_task_detail_shown().unwrap(), "task-2");
    }
}
