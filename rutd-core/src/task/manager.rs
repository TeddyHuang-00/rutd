use anyhow::{Context, Result};
use chrono::{DateTime, Local};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use uuid::Uuid;

use super::{
    active_task::{self, ActiveTask},
    filter::{DateRange, FilterOptions},
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
    fn matches_filters(task: &Task, filter_options: &FilterOptions) -> bool {
        // Check basic filters
        if let Some(p) = &filter_options.priority {
            if task.priority != *p {
                return false;
            }
        }
        if let Some(s) = &filter_options.scope {
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
    pub fn new(path_config: PathConfig, git_config: GitConfig) -> Self {
        TaskManager {
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
        storage::save_task(&self.path_config.task_dir(), &task, "create", "Create task")?;
        Ok(id)
    }

    /// List tasks with filtering support
    pub fn list_tasks(&self, filter_options: &FilterOptions) -> Result<Vec<Task>> {
        let tasks = storage::load_all_tasks(&self.path_config.task_dir())?;
        let filtered_tasks = tasks
            .into_iter()
            .filter(|task| Self::matches_filters(task, filter_options))
            .collect::<Vec<Task>>();

        Ok(filtered_tasks)
    }

    /// Mark a task as completed
    pub fn mark_task_done(&self, task_id: &str) -> Result<()> {
        let mut task = storage::load_task(&self.path_config.task_dir(), task_id)?;

        // Check if the task is already done
        if task.status == TaskStatus::Done {
            anyhow::bail!("Task is already completed");
        }

        // Check if this is the active task
        let is_active_task =
            match active_task::load_active_task(&self.path_config.active_task_file())? {
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
            &self.path_config.task_dir(),
            &task,
            "finish",
            "Mark task as done",
        )?;

        // If this was the active task, clear the active task record
        if is_active_task {
            active_task::clear_active_task(&self.path_config.active_task_file())?;
            log::debug!("Completed active task: {task_id} and cleared active task file");
        } else {
            log::debug!("Completed task: {task_id}");
        }

        Ok(())
    }

    /// Start working on a task
    pub fn start_task(&self, task_id: &str) -> Result<String> {
        // Check if there is already an active task
        if let Some(active) = active_task::load_active_task(&self.path_config.active_task_file())? {
            let active_task_obj =
                storage::load_task(&self.path_config.task_dir(), &active.task_id)?;
            anyhow::bail!(
                "There's already an active task: {} - {}. Stop it first.",
                active.task_id,
                active_task_obj.description
            )
        }

        // Load task
        let task = storage::load_task(&self.path_config.task_dir(), task_id)?;

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
        active_task::save_active_task(&self.path_config.active_task_file(), &active)?;

        log::debug!("Started task: {} and saved to active task file", task.id);
        Ok(task.id)
    }

    /// Stop working on a task
    pub fn stop_task(&self) -> Result<String> {
        // Check if there's an active task
        let Some(active_task_info) =
            active_task::load_active_task(&self.path_config.active_task_file())?
        else {
            // No active task found
            anyhow::bail!("No active task found. Task might not be in progress.")
        };

        // Load the task
        let mut task = storage::load_task(&self.path_config.task_dir(), &active_task_info.task_id)?;

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
            &self.path_config.task_dir(),
            &task,
            "update",
            "Update time spent on task",
        )?;

        // Clear the active task record
        active_task::clear_active_task(&self.path_config.active_task_file())?;

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
                    active_task::load_active_task(&self.path_config.active_task_file())?
                else {
                    anyhow::bail!("No active task found");
                };
                active_task.task_id
            }
        };
        let mut task = storage::load_task(&self.path_config.task_dir(), &task_id)?;

        // Check if the task is already done or aborted
        if task.status == TaskStatus::Done {
            anyhow::bail!("Cannot abort a completed task");
        }
        if task.status == TaskStatus::Aborted {
            anyhow::bail!("Task is already aborted");
        }

        // Check if this is the active task
        let is_active_task =
            match active_task::load_active_task(&self.path_config.active_task_file())? {
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
        storage::save_task(&self.path_config.task_dir(), &task, "cancel", "Cancel task")?;

        // If this was the active task, clear the active task record
        if is_active_task {
            active_task::clear_active_task(&self.path_config.active_task_file())?;
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
        let mut task = storage::load_task(&self.path_config.task_dir(), task_id)?;

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
                &self.path_config.task_dir(),
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
        filter_options: &FilterOptions,
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
            &self.path_config.task_dir(),
            &tasks
                .iter()
                .map(|task| task.id.as_str())
                .collect::<Vec<_>>(),
        )?;

        Ok(count)
    }

    /// Clone a remote repository
    pub fn clone_repo(&self, url: &str) -> Result<()> {
        GitRepo::clone(self.path_config.task_dir(), url, &self.git_config)?;
        Ok(())
    }

    /// Sync with remote repository
    pub fn sync(&self, prefer: MergeStrategy) -> Result<()> {
        let git_repo = GitRepo::init(self.path_config.task_dir())?;
        git_repo.sync(prefer, &self.git_config)?;
        Ok(())
    }
}
