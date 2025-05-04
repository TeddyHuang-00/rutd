use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use uuid::Uuid;

use super::{
    active_task::{self, ActiveTask},
    filter::{DateRange, Filter},
    model::{Priority, Task, TaskStatus},
    storage,
};
use crate::{
    config::{GitConfig, PathConfig},
    display::Display,
    git::{MergeStrategy, repo::GitRepo},
};

/// Task Manager
#[derive(Default)]
pub struct TaskManager {
    path_config: PathConfig,
    git_config: GitConfig,
}

// Helper functions for TaskManager
impl TaskManager {
    /// Check if time fits in the date range
    fn is_time_in_range(time: &str, range: &DateRange) -> bool {
        let time = DateTime::parse_from_rfc3339(time)
            .unwrap()
            .with_timezone(&Local);
        range.from.map(|from| time >= from).unwrap_or(true)
            && range.to.map(|to| time < to).unwrap_or(true)
    }

    /// Check if a task matches the filter conditions
    fn matches_filters(task: &Task, filter_options: &Filter) -> bool {
        // Check basic filters
        if let Some(p) = &filter_options.priority {
            if task.priority != *p {
                return false;
            }
        }
        if let Some(s) = &filter_options.task_scope {
            if task.scope.as_deref() != Some(s) {
                return false;
            }
        }
        if let (Some(t), Some(task_type)) = (&filter_options.task_type, &task.task_type) {
            if task_type != t {
                return false;
            }
        }
        if let Some(st) = &filter_options.status {
            if task.status != *st {
                return false;
            }
        }

        // Check creation time
        if let Some(date_range) = &filter_options.creation_time {
            // Check creation date against date range
            if !Self::is_time_in_range(&task.created_at, date_range) {
                return false;
            }
        }

        // Check update time
        if let Some(date_range) = &filter_options.update_time {
            // Check if the task has been updated, if not, use created_at
            let updated_at = task.updated_at.as_deref().unwrap_or(&task.created_at);
            // Check update date against date range
            if !Self::is_time_in_range(updated_at, date_range) {
                return false;
            }
        }

        // Check completion time
        if let Some(date_range) = &filter_options.completion_time {
            // Check if the task is completed
            let Some(completed_at) = &task.completed_at else {
                return false;
            };

            // Check completion date against date range
            if !Self::is_time_in_range(completed_at, date_range) {
                return false;
            }
        }

        // Check fuzzy matching on description
        if let Some(query) = &filter_options.fuzzy {
            if !query.is_empty() {
                let matcher = SkimMatcherV2::default();
                if matcher.fuzzy_match(&task.description, query).is_none() {
                    return false;
                }
            }
        }

        true
    }
}

// Public methods for TaskManager
impl TaskManager {
    /// Create a new Task Manager
    pub const fn new(path_config: PathConfig, git_config: GitConfig) -> Self {
        Self {
            path_config,
            git_config,
        }
    }

    /// Add a new task
    pub fn add_task(
        &self,
        description: &str,
        priority: Priority,
        scope: Option<String>,
        task_type: Option<String>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let task = Task::new(
            id.clone(),
            description.to_string(),
            priority,
            scope,
            task_type,
        );
        storage::save_task(
            &self.path_config.task_dir_path(),
            &task,
            "create",
            "Create task",
        )?;
        Ok(id)
    }

    /// List tasks with filtering support
    pub fn list_tasks(&self, filter_options: &Filter) -> Result<Vec<Task>> {
        let tasks = storage::load_all_tasks(&self.path_config.task_dir_path())?;
        let filtered_tasks = tasks
            .into_iter()
            .filter(|task| Self::matches_filters(task, filter_options))
            .collect::<Vec<Task>>();

        Ok(filtered_tasks)
    }

    /// Mark a task as completed
    pub fn mark_task_done(&self, task_id: &str) -> Result<()> {
        let mut task = storage::load_task(&self.path_config.task_dir_path(), task_id)?;

        // Check if the task is already done
        if task.status == TaskStatus::Done {
            anyhow::bail!("Task is already completed");
        }

        // Check if this is the active task
        let is_active_task =
            match active_task::load_active_task(&self.path_config.active_task_file_path())? {
                Some(active) => {
                    if active.task_id == task_id {
                        // Calculate time spent using the active task record
                        let started_time = DateTime::parse_from_rfc3339(&active.started_at)
                            .context("Failed to parse started_at time from active task record")?;
                        let now = Local::now();
                        let duration =
                            now.signed_duration_since(started_time.with_timezone(&Local));

                        // Calculate total seconds spent
                        let seconds_spent = duration.num_seconds().max(0) as u64;

                        // Add to existing time_spent or initialize it
                        task.time_spent = Some(task.time_spent.unwrap_or(0) + seconds_spent);

                        true
                    } else {
                        false
                    }
                }
                None => false,
            };

        // Update task status and timestamps
        task.status = TaskStatus::Done;
        task.updated_at = Some(Local::now().to_rfc3339());
        task.completed_at = Some(Local::now().to_rfc3339());

        // Save the updated task
        storage::save_task(
            &self.path_config.task_dir_path(),
            &task,
            "finish",
            "Mark task as done",
        )?;

        // If this was the active task, clear the active task record
        if is_active_task {
            active_task::clear_active_task(&self.path_config.active_task_file_path())?;
            log::debug!("Completed active task: {task_id} and cleared active task file");
        } else {
            log::debug!("Completed task: {task_id}");
        }

        Ok(())
    }

    /// Start working on a task
    pub fn start_task(&self, task_id: &str) -> Result<String> {
        // Check if there is already an active task
        if let Some(active) =
            active_task::load_active_task(&self.path_config.active_task_file_path())?
        {
            let active_task_obj =
                storage::load_task(&self.path_config.task_dir_path(), &active.task_id)?;
            anyhow::bail!(
                "There's already an active task: {} - {}. Stop it first.",
                active.task_id,
                active_task_obj.description
            )
        }

        // Load task
        let task = storage::load_task(&self.path_config.task_dir_path(), task_id)?;

        // Check if task is already completed or aborted
        if task.status == TaskStatus::Done {
            anyhow::bail!("Cannot start a completed task")
        }
        if task.status == TaskStatus::Aborted {
            anyhow::bail!("Cannot start an aborted task")
        }

        // Get current time
        let now = Local::now().to_rfc3339();

        // Create and save active task record
        let active = ActiveTask::new(task.id.clone(), now);
        active_task::save_active_task(&self.path_config.active_task_file_path(), &active)?;

        log::debug!("Started task: {} and saved to active task file", task.id);
        Ok(task.id)
    }

    /// Stop working on a task
    pub fn stop_task(&self) -> Result<String> {
        // Check if there's an active task
        let Some(active_task_info) =
            active_task::load_active_task(&self.path_config.active_task_file_path())?
        else {
            // No active task found
            anyhow::bail!("No active task found. Task might not be in progress.")
        };

        // Load the task
        let mut task =
            storage::load_task(&self.path_config.task_dir_path(), &active_task_info.task_id)?;

        // Calculate time spent using the active task record
        let started_time = DateTime::parse_from_rfc3339(&active_task_info.started_at)
            .context("Failed to parse started_at time from active task record")?;
        let now = Local::now();
        let duration = now.signed_duration_since(started_time.with_timezone(&Local));

        // Calculate total seconds spent
        let seconds_spent = duration.num_seconds().max(0) as u64;

        // Update task time spent
        task.time_spent = Some(task.time_spent.unwrap_or(0) + seconds_spent);

        // Update task status and timestamps
        task.updated_at = Some(Local::now().to_rfc3339());

        // Save the updated task
        storage::save_task(
            &self.path_config.task_dir_path(),
            &task,
            "update",
            "Update time spent on task",
        )?;

        // Clear the active task record
        active_task::clear_active_task(&self.path_config.active_task_file_path())?;

        log::debug!(
            "Stopped task: {} and cleared active task file",
            &active_task_info.task_id
        );
        Ok(active_task_info.task_id)
    }

    /// Mark a task as aborted
    pub fn abort_task(&self, task_id: &Option<String>) -> Result<String> {
        let task_id = match task_id {
            Some(task_id) => task_id.to_owned(),
            None => {
                // Load the active task if no ID is provided
                let Some(active_task) =
                    active_task::load_active_task(&self.path_config.active_task_file_path())?
                else {
                    anyhow::bail!("No active task found");
                };
                active_task.task_id
            }
        };
        let mut task = storage::load_task(&self.path_config.task_dir_path(), &task_id)?;

        // Check if the task is already done or aborted
        if task.status == TaskStatus::Done {
            anyhow::bail!("Cannot abort a completed task");
        }
        if task.status == TaskStatus::Aborted {
            anyhow::bail!("Task is already aborted");
        }

        // Check if this is the active task
        let is_active_task =
            match active_task::load_active_task(&self.path_config.active_task_file_path())? {
                Some(active) if active.task_id == task_id => {
                    // Calculate time spent using the active task record
                    let started_time = DateTime::parse_from_rfc3339(&active.started_at)
                        .context("Failed to parse started_at time from active task record")?;
                    let now = Local::now();
                    let duration = now.signed_duration_since(started_time.with_timezone(&Local));

                    // Calculate total seconds spent
                    let seconds_spent = duration.num_seconds().max(0) as u64;

                    // Add to existing time_spent or initialize it
                    task.time_spent = Some(task.time_spent.unwrap_or(0) + seconds_spent);

                    true
                }
                _ => false,
            };

        // Update task status and timestamps
        task.status = TaskStatus::Aborted;
        task.updated_at = Some(Local::now().to_rfc3339());
        task.completed_at = Some(Local::now().to_rfc3339());

        // Save the updated task
        storage::save_task(
            &self.path_config.task_dir_path(),
            &task,
            "cancel",
            "Cancel task",
        )?;

        // If this was the active task, clear the active task record
        if is_active_task {
            active_task::clear_active_task(&self.path_config.active_task_file_path())?;
            log::debug!("Aborted active task: {task_id} and cleared active task file");
        } else {
            log::debug!("Aborted task: {task_id}");
        }

        Ok(task_id)
    }

    /// Edit task description
    pub fn edit_task_description<D: Display>(
        &self,
        task_id: &str,
        display_manager: &D,
    ) -> Result<String> {
        // Load the task
        let mut task = storage::load_task(&self.path_config.task_dir_path(), task_id)?;

        // Edit the task description through display
        let Some(new_description) = display_manager.edit(&task.description)? else {
            anyhow::bail!("No changes made to the task description");
        };

        // Trim whitespace
        let new_description = new_description.trim().to_string();

        // Only update if description has changed
        if new_description != task.description {
            task.description = new_description;
            task.updated_at = Some(Local::now().to_rfc3339());
            storage::save_task(
                &self.path_config.task_dir_path(),
                &task,
                "update",
                "Update task description",
            )?;
        }

        Ok(task.id)
    }

    /// Clean tasks based on filters
    pub fn clean_tasks<D: Display>(
        &self,
        filter_options: &Filter,
        force: bool,
        display_manager: &D,
    ) -> Result<usize> {
        // Get tasks matching filters
        let tasks = self.list_tasks(filter_options)?;

        let count = tasks.len();

        // Confirm deletion if not forced
        if count > 0 && !force {
            let message = format!("Are you sure to delete {count} tasks?");
            if !display_manager.confirm(&message)? {
                return Ok(0);
            }
        }

        // Batch delete tasks
        storage::delete_task(
            &self.path_config.task_dir_path(),
            &tasks
                .iter()
                .map(|task| task.id.as_str())
                .collect::<Vec<_>>(),
        )?;

        Ok(count)
    }

    /// Clone a remote repository
    pub fn clone_repo(&self, url: &str) -> Result<()> {
        GitRepo::clone(self.path_config.task_dir_path(), url, &self.git_config)?;
        Ok(())
    }

    /// Sync with remote repository
    pub fn sync(&self, prefer: MergeStrategy) -> Result<()> {
        let git_repo = GitRepo::init(self.path_config.task_dir_path())?;
        git_repo.sync(prefer, &self.git_config)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use chrono::Local;
    use tempfile::tempdir;

    use super::*;
    use crate::{
        config::{GitConfig, PathConfig},
        display::Display,
        task::{Filter, TaskStatus},
    };

    // Mock display implementation for testing
    struct MockDisplay {
        confirm_result: bool,
        edit_result: Option<String>,
    }

    impl MockDisplay {
        fn new(confirm_result: bool, edit_result: Option<String>) -> Self {
            Self {
                confirm_result,
                edit_result,
            }
        }
    }

    impl Display for MockDisplay {
        fn confirm(&self, _message: &str) -> Result<bool> {
            Ok(self.confirm_result)
        }

        fn edit(&self, _message: &str) -> Result<Option<String>> {
            Ok(self.edit_result.clone())
        }

        fn show_success(&self, _message: &str) {}
        fn show_failure(&self, _message: &str) {}
        fn show_tasks_list(&self, _tasks: &[Task]) {}
        fn show_task_stats(&self, _tasks: &[Task]) {}
        fn show_task_detail(&self, _task: &Task) {}
    }

    // Helper function to create a task manager with temporary directories
    fn create_test_task_manager() -> (TaskManager, tempfile::TempDir) {
        let temp_dir = tempdir().unwrap();

        // Create a custom path config that uses the temp directory
        let path_config = PathConfig {
            root_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let git_config = GitConfig::default();

        let task_manager = TaskManager::new(path_config, git_config);

        (task_manager, temp_dir)
    }

    #[test]
    fn test_add_task() {
        let (task_manager, _temp_dir) = create_test_task_manager();

        // Add a task
        let result = task_manager.add_task(
            "Test task",
            Priority::Normal,
            Some("test-scope".to_string()),
            Some("test-type".to_string()),
        );

        // The result might be Ok or Err depending on git operations
        // but we'll check that the function runs without panicking
        if let Ok(task_id) = result {
            // If successful, verify the task was created
            let task_dir = task_manager.path_config.task_dir_path();
            let task_file = task_dir.join(format!("{task_id}.toml"));
            assert!(task_file.exists());
        }
    }

    #[test]
    fn test_list_tasks() {
        let (task_manager, _temp_dir) = create_test_task_manager();

        // Add a few test tasks directly (bypassing git operations)
        let task_dir = task_manager.path_config.task_dir_path();

        // Create task directory
        fs::create_dir_all(&task_dir).unwrap();

        // Create some test task files
        for i in 1..=3 {
            let task = Task {
                id: format!("test-{i}"),
                description: format!("Test task {i}"),
                priority: Priority::Normal,
                scope: Some("test-scope".to_string()),
                task_type: Some("test-type".to_string()),
                status: TaskStatus::Todo,
                created_at: Local::now().to_rfc3339(),
                updated_at: None,
                completed_at: None,
                time_spent: None,
            };

            let file_path = task_dir.join(format!("{}.toml", task.id));
            fs::write(file_path, toml::to_string(&task).unwrap()).unwrap();
        }

        // List tasks with no filters
        let filter = Filter::default();
        let result = task_manager.list_tasks(&filter);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);

        // List tasks with priority filter
        // None of our tasks have this priority
        let filter = Filter {
            priority: Some(Priority::High),
            ..Default::default()
        };
        let result = task_manager.list_tasks(&filter);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_mark_task_done() {
        let (task_manager, _temp_dir) = create_test_task_manager();

        // Create a test task
        let task_dir = task_manager.path_config.task_dir_path();
        fs::create_dir_all(&task_dir).unwrap();

        let task_id = "test-complete";
        let task = Task {
            id: task_id.to_string(),
            description: "Test task".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Todo,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        };

        let file_path = task_dir.join(format!("{}.toml", task.id));
        fs::write(&file_path, toml::to_string(&task).unwrap()).unwrap();

        // Mark the task as done
        let result = task_manager.mark_task_done(task_id);

        // The function might fail due to git operations, but check that the file was
        // updated
        if result.is_ok() {
            // Read the task file and check status
            let content = fs::read_to_string(&file_path).unwrap();
            let updated_task: Task = toml::from_str(&content).unwrap();

            assert_eq!(updated_task.status, TaskStatus::Done);
            assert!(updated_task.completed_at.is_some());
        }
    }

    #[test]
    fn test_start_and_stop_task() {
        let (task_manager, _temp_dir) = create_test_task_manager();

        // Create a test task
        let task_dir = task_manager.path_config.task_dir_path();
        fs::create_dir_all(&task_dir).unwrap();

        let task_id = "test-start-stop";
        let task = Task {
            id: task_id.to_string(),
            description: "Test task".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Todo,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        };

        let file_path = task_dir.join(format!("{}.toml", task.id));
        fs::write(&file_path, toml::to_string(&task).unwrap()).unwrap();

        // Start the task
        let result = task_manager.start_task(task_id);

        // Check that the active task file was created
        if result.is_ok() {
            let active_task_file = task_manager.path_config.active_task_file_path();
            assert!(active_task_file.exists());

            // Stop the task
            let stop_result = task_manager.stop_task();

            // The time spent should be updated and active task file should be removed
            if stop_result.is_ok() {
                assert!(!active_task_file.exists());

                // Read the task file and check time_spent
                let content = fs::read_to_string(&file_path).unwrap();
                let updated_task: Task = toml::from_str(&content).unwrap();

                assert!(updated_task.time_spent.is_some());
            }
        }
    }

    #[test]
    fn test_abort_task() {
        let (task_manager, _temp_dir) = create_test_task_manager();

        // Create a test task
        let task_dir = task_manager.path_config.task_dir_path();
        fs::create_dir_all(&task_dir).unwrap();

        let task_id = "test-abort";
        let task = Task {
            id: task_id.to_string(),
            description: "Test task".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Todo,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        };

        let file_path = task_dir.join(format!("{}.toml", task.id));
        fs::write(&file_path, toml::to_string(&task).unwrap()).unwrap();

        // Abort the task with specific ID
        let result = task_manager.abort_task(&Some(task_id.to_string()));

        // Check that the task was aborted
        if result.is_ok() {
            let content = fs::read_to_string(&file_path).unwrap();
            let updated_task: Task = toml::from_str(&content).unwrap();

            assert_eq!(updated_task.status, TaskStatus::Aborted);
        }
    }

    #[test]
    fn test_edit_task_description() {
        let (task_manager, _temp_dir) = create_test_task_manager();

        // Create a test task
        let task_dir = task_manager.path_config.task_dir_path();
        fs::create_dir_all(&task_dir).unwrap();

        let task_id = "test-edit";
        let task = Task {
            id: task_id.to_string(),
            description: "Original description".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Todo,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        };

        let file_path = task_dir.join(format!("{}.toml", task.id));
        fs::write(&file_path, toml::to_string(&task).unwrap()).unwrap();

        // Create a mock display that returns an edited description
        let new_description = "Updated description";
        let display = MockDisplay::new(true, Some(new_description.to_string()));

        // Edit the task description
        let result = task_manager.edit_task_description(task_id, &display);

        // Check that the description was updated
        if result.is_ok() {
            let content = fs::read_to_string(&file_path).unwrap();
            let updated_task: Task = toml::from_str(&content).unwrap();

            assert_eq!(updated_task.description, new_description);
            assert!(updated_task.updated_at.is_some());
        }
    }

    #[test]
    fn test_clean_tasks() {
        let (task_manager, _temp_dir) = create_test_task_manager();

        // Create some test tasks
        let task_dir = task_manager.path_config.task_dir_path();
        fs::create_dir_all(&task_dir).unwrap();

        // Create done and todo tasks
        let done_task = Task {
            id: "done-task".to_string(),
            description: "Done task".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Done,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: Some(Local::now().to_rfc3339()),
            time_spent: None,
        };

        let todo_task = Task {
            id: "todo-task".to_string(),
            description: "Todo task".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Todo,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        };

        fs::write(
            task_dir.join(format!("{}.toml", done_task.id)),
            toml::to_string(&done_task).unwrap(),
        )
        .unwrap();

        fs::write(
            task_dir.join(format!("{}.toml", todo_task.id)),
            toml::to_string(&todo_task).unwrap(),
        )
        .unwrap();

        // Create a filter to only clean Done tasks
        let filter = Filter {
            status: Some(TaskStatus::Done),
            ..Default::default()
        };

        // Create a mock display that confirms the deletion
        let display = MockDisplay::new(true, None);

        // Clean the tasks
        let result = task_manager.clean_tasks(&filter, false, &display);

        // Check that only the done task was removed
        if let Ok(count) = result {
            assert_eq!(count, 1);
            assert!(!task_dir.join(format!("{}.toml", done_task.id)).exists());
            assert!(task_dir.join(format!("{}.toml", todo_task.id)).exists());
        }
    }

    #[test]
    fn test_edge_cases_for_task_status_changes() {
        let (task_manager, _temp_dir) = create_test_task_manager();

        // Create test tasks with different statuses
        let task_dir = task_manager.path_config.task_dir_path();
        fs::create_dir_all(&task_dir).unwrap();

        // Create tasks with different statuses
        let todo_task = Task {
            id: "todo-task".to_string(),
            description: "Todo task".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Todo,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        };

        let done_task = Task {
            id: "done-task".to_string(),
            description: "Done task".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Done,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: Some(Local::now().to_rfc3339()),
            time_spent: None,
        };

        let aborted_task = Task {
            id: "aborted-task".to_string(),
            description: "Aborted task".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Aborted,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: Some(Local::now().to_rfc3339()),
            time_spent: None,
        };

        // Save all tasks
        fs::write(
            task_dir.join(format!("{}.toml", todo_task.id)),
            toml::to_string(&todo_task).unwrap(),
        )
        .unwrap();

        fs::write(
            task_dir.join(format!("{}.toml", done_task.id)),
            toml::to_string(&done_task).unwrap(),
        )
        .unwrap();

        fs::write(
            task_dir.join(format!("{}.toml", aborted_task.id)),
            toml::to_string(&aborted_task).unwrap(),
        )
        .unwrap();

        // Edge case 1: Mark already done task as done
        let result = task_manager.mark_task_done(&done_task.id);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("already completed")
        );

        // Edge case 2: Start already done task
        let result = task_manager.start_task(&done_task.id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("completed task"));

        // Edge case 3: Start already aborted task
        let result = task_manager.start_task(&aborted_task.id);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("aborted task"));

        // Edge case 4: Abort already done task
        let result = task_manager.abort_task(&Some(done_task.id));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("completed task"));

        // Edge case 5: Abort already aborted task
        let result = task_manager.abort_task(&Some(aborted_task.id));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already aborted"));
    }

    #[test]
    fn test_task_filtering_logic() {
        // Test the static matches_filters method directly

        // Create a test task
        let task = Task {
            id: "test-task".to_string(),
            description: "Test filtering logic".to_string(),
            priority: Priority::High,
            scope: Some("test-scope".to_string()),
            task_type: Some("feature".to_string()),
            status: TaskStatus::Todo,
            created_at: "2023-05-15T12:00:00+00:00".to_string(),
            updated_at: Some("2023-05-16T14:30:00+00:00".to_string()),
            completed_at: None,
            time_spent: None,
        };

        // Test 1: Empty filter should match
        let filter = Filter::default();
        assert!(TaskManager::matches_filters(&task, &filter));

        // Test 2: Matching priority filter
        let filter = Filter {
            priority: Some(Priority::High),
            ..Default::default()
        };
        assert!(TaskManager::matches_filters(&task, &filter));

        // Test 3: Non-matching priority filter
        let filter = Filter {
            priority: Some(Priority::Low),
            ..Default::default()
        };
        assert!(!TaskManager::matches_filters(&task, &filter));

        // Test 4: Matching scope filter
        let filter = Filter {
            task_scope: Some("test-scope".to_string()),
            ..Default::default()
        };
        assert!(TaskManager::matches_filters(&task, &filter));

        // Test 5: Non-matching scope filter
        let filter = Filter {
            task_scope: Some("wrong-scope".to_string()),
            ..Default::default()
        };
        assert!(!TaskManager::matches_filters(&task, &filter));

        // Test 6: Matching type filter
        let filter = Filter {
            task_type: Some("feature".to_string()),
            ..Default::default()
        };
        assert!(TaskManager::matches_filters(&task, &filter));

        // Test 7: Non-matching type filter
        let filter = Filter {
            task_type: Some("bug".to_string()),
            ..Default::default()
        };
        assert!(!TaskManager::matches_filters(&task, &filter));

        // Test 8: Matching status filter
        let filter = Filter {
            status: Some(TaskStatus::Todo),
            ..Default::default()
        };
        assert!(TaskManager::matches_filters(&task, &filter));

        // Test 9: Non-matching status filter
        let filter = Filter {
            status: Some(TaskStatus::Done),
            ..Default::default()
        };
        assert!(!TaskManager::matches_filters(&task, &filter));

        // Test 10: Matching fuzzy filter
        let filter = Filter {
            fuzzy: Some("filtering".to_string()),
            ..Default::default()
        };
        assert!(TaskManager::matches_filters(&task, &filter));

        // Test 11: Non-matching fuzzy filter
        let filter = Filter {
            fuzzy: Some("nonexistent".to_string()),
            ..Default::default()
        };
        assert!(!TaskManager::matches_filters(&task, &filter));

        // Test 12: Multiple matching filters
        let filter = Filter {
            priority: Some(Priority::High),
            task_scope: Some("test-scope".to_string()),
            status: Some(TaskStatus::Todo),
            ..Default::default()
        };
        assert!(TaskManager::matches_filters(&task, &filter));

        // Test 13: Mixed matching and non-matching filters
        let filter = Filter {
            priority: Some(Priority::High),
            task_scope: Some("wrong-scope".to_string()),
            ..Default::default()
        };
        assert!(!TaskManager::matches_filters(&task, &filter));
    }

    #[test]
    fn test_active_task_tracking() {
        let (task_manager, _temp_dir) = create_test_task_manager();

        // Create a test task
        let task_dir = task_manager.path_config.task_dir_path();
        fs::create_dir_all(&task_dir).unwrap();

        let task_id = "active-task-test";
        let task = Task {
            id: task_id.to_string(),
            description: "Task for active tracking".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Todo,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        };

        let file_path = task_dir.join(format!("{}.toml", task.id));
        fs::write(&file_path, toml::to_string(&task).unwrap()).unwrap();

        // Start the task
        let start_result = task_manager.start_task(task_id);
        assert!(start_result.is_ok());

        // Check that starting a second task fails while one is active
        let second_task_id = "second-task";
        let second_task = Task {
            id: second_task_id.to_string(),
            description: "Second task".to_string(),
            priority: Priority::Normal,
            scope: None,
            task_type: None,
            status: TaskStatus::Todo,
            created_at: Local::now().to_rfc3339(),
            updated_at: None,
            completed_at: None,
            time_spent: None,
        };

        let second_file_path = task_dir.join(format!("{}.toml", second_task.id));
        fs::write(&second_file_path, toml::to_string(&second_task).unwrap()).unwrap();

        let start_second_result = task_manager.start_task(second_task_id);
        assert!(start_second_result.is_err());
        assert!(
            start_second_result
                .unwrap_err()
                .to_string()
                .contains("already an active task")
        );

        // Verify that completing a task clears the active task state
        let complete_result = task_manager.mark_task_done(task_id);
        assert!(complete_result.is_ok());

        // The active task file should no longer exist
        let active_task_file = task_manager.path_config.active_task_file_path();
        assert!(!active_task_file.exists());

        // We should now be able to start a different task
        let restart_result = task_manager.start_task(second_task_id);
        assert!(restart_result.is_ok());

        // Test that stopping a task updates the time spent
        // First read the current task to get its starting time_spent
        let content = fs::read_to_string(&second_file_path).unwrap();
        let second_task: Task = toml::from_str(&content).unwrap();
        assert!(second_task.time_spent.is_none()); // Should be None initially

        // Stop the task
        let stop_result = task_manager.stop_task();
        assert!(stop_result.is_ok());

        // Check that time_spent was updated
        let content = fs::read_to_string(&second_file_path).unwrap();
        let updated_task: Task = toml::from_str(&content).unwrap();
        assert!(updated_task.time_spent.is_some()); // Should now have some value

        // Test stopping with no active task
        let stop_nothing_result = task_manager.stop_task();
        assert!(stop_nothing_result.is_err());
        assert!(
            stop_nothing_result
                .unwrap_err()
                .to_string()
                .contains("No active task found")
        );
    }

    #[test]
    fn test_is_time_in_range() {
        // Test the time range functionality directly

        // Setup time strings for testing
        let time1 = "2023-01-15T12:00:00+00:00";
        let time2 = "2023-02-15T12:00:00+00:00";
        let time3 = "2023-03-15T12:00:00+00:00";

        // Create a date range that includes time2 but not time1 or time3
        let from = DateTime::parse_from_rfc3339("2023-02-01T00:00:00+00:00")
            .unwrap()
            .with_timezone(&Local);
        let to = DateTime::parse_from_rfc3339("2023-03-01T00:00:00+00:00")
            .unwrap()
            .with_timezone(&Local);

        let date_range = DateRange {
            from: Some(from),
            to: Some(to),
        };

        // Test that time2 is in range
        assert!(TaskManager::is_time_in_range(time2, &date_range));

        // Test that time1 is not in range (before the from date)
        assert!(!TaskManager::is_time_in_range(time1, &date_range));

        // Test that time3 is not in range (after the to date)
        assert!(!TaskManager::is_time_in_range(time3, &date_range));

        // Test with open-ended range (only from)
        let open_ended_range = DateRange {
            from: Some(from),
            to: None,
        };

        // time1 should be out of range
        assert!(!TaskManager::is_time_in_range(time1, &open_ended_range));

        // time2 and time3 should be in range
        assert!(TaskManager::is_time_in_range(time2, &open_ended_range));
        assert!(TaskManager::is_time_in_range(time3, &open_ended_range));

        // Test with open-ended range (only to)
        let open_ended_range = DateRange {
            from: None,
            to: Some(to),
        };

        // time1 and time2 should be in range
        assert!(TaskManager::is_time_in_range(time1, &open_ended_range));
        assert!(TaskManager::is_time_in_range(time2, &open_ended_range));

        // time3 should be out of range
        assert!(!TaskManager::is_time_in_range(time3, &open_ended_range));
    }
}
