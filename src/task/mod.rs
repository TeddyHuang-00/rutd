pub mod active_task;
pub mod model;
pub mod storage;

use std::{
    io::{Read, Write},
    path::PathBuf,
    process::Command,
};

use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Duration, Utc};
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};
use log::debug;
pub use model::{Priority, Task, TaskStatus};
use shellexpand::tilde;
use uuid::Uuid;

use crate::{
    display::DisplayManager,
    git::{MergeStrategy, repo::GitRepo},
    task::active_task::ActiveTask,
};

const TASKS_DIR: &str = "~/.rutd/tasks";

/// Task Manager
pub struct TaskManager {
    tasks_dir: PathBuf,
}

impl Default for TaskManager {
    fn default() -> Self {
        TaskManager::new(tilde(TASKS_DIR).as_ref())
    }
}

impl TaskManager {
    /// Create a new Task Manager
    pub fn new(tasks_dir: &str) -> Self {
        TaskManager {
            tasks_dir: tasks_dir.into(),
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
        storage::save_task(&self.tasks_dir, &task)?;
        Ok(id)
    }

    /// List tasks with filtering support
    pub fn list_tasks(
        &self,
        priority_filter: Option<Priority>,
        scope_filter: Option<&str>,
        type_filter: Option<String>,
        status_filter: Option<TaskStatus>,
        from_date: Option<&str>,
        to_date: Option<&str>,
        fuzzy_query: Option<&str>,
        show_stats: bool,
    ) -> Result<Vec<Task>> {
        let tasks = storage::load_all_tasks(&self.tasks_dir)?;
        let filtered_tasks = tasks
            .into_iter()
            .filter(|task| {
                Self::matches_filters(
                    task,
                    &priority_filter,
                    &scope_filter,
                    &type_filter,
                    &status_filter,
                    &from_date,
                    &to_date,
                    &fuzzy_query,
                )
            })
            .collect::<Vec<Task>>();

        // 不再在这里显示统计信息，统计信息的显示已移至DisplayManager

        Ok(filtered_tasks)
    }

    /// Check if a task matches the filter conditions
    fn matches_filters(
        task: &Task,
        priority_filter: &Option<Priority>,
        scope_filter: &Option<&str>,
        type_filter: &Option<String>,
        status_filter: &Option<TaskStatus>,
        from_date: &Option<&str>,
        to_date: &Option<&str>,
        fuzzy_query: &Option<&str>,
    ) -> bool {
        // Check basic filters
        if let Some(p) = priority_filter {
            if task.priority != *p {
                return false;
            }
        }
        if let Some(s) = scope_filter {
            if task.scope.as_deref() != Some(s) {
                return false;
            }
        }
        if let (Some(t), Some(task_type)) = (type_filter, &task.task_type) {
            if task_type != t {
                return false;
            }
        }
        if let Some(st) = status_filter {
            if task.status != *st {
                return false;
            }
        }

        // Check date range filters
        if let Some(from) = from_date {
            if let Some(completed_at) = &task.completed_at {
                if let (Ok(from_date), Ok(completed_date)) = (
                    DateTime::parse_from_rfc3339(from),
                    DateTime::parse_from_rfc3339(completed_at),
                ) {
                    if completed_date < from_date {
                        return false;
                    }
                }
            } else if task.status == TaskStatus::Done || task.status == TaskStatus::Aborted {
                // If the task is completed but has no completion date, exclude it
                return false;
            }
        }

        if let Some(to) = to_date {
            if let Some(completed_at) = &task.completed_at {
                if let (Ok(to_date), Ok(completed_date)) = (
                    DateTime::parse_from_rfc3339(to),
                    DateTime::parse_from_rfc3339(completed_at),
                ) {
                    if completed_date > to_date {
                        return false;
                    }
                }
            }
        }

        // Check fuzzy matching on description
        if let Some(query) = fuzzy_query {
            if !query.is_empty() {
                let matcher = SkimMatcherV2::default();
                if matcher.fuzzy_match(&task.description, query).is_none() {
                    return false;
                }
            }
        }

        true
    }

    /// Mark a task as completed
    pub fn mark_task_done(&self, task_id: &str) -> Result<()> {
        let mut task = storage::load_task(&self.tasks_dir, task_id)?;

        // Check if the task is already done
        if task.status == TaskStatus::Done {
            return Err(anyhow!("Task is already completed"));
        }

        // Check if this is the active task
        let is_active_task = match active_task::load_active_task(&self.tasks_dir)? {
            Some(active) => {
                if active.task_id == task_id {
                    // Calculate time spent using the active task record
                    let started_time = DateTime::parse_from_rfc3339(&active.started_at)
                        .context("Failed to parse started_at time from active task record")?;
                    let now = Utc::now();
                    let duration = now.signed_duration_since(started_time.with_timezone(&Utc));

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
        task.updated_at = Some(chrono::Utc::now().to_rfc3339());
        task.completed_at = Some(chrono::Utc::now().to_rfc3339());

        // Save the updated task
        storage::save_task(&self.tasks_dir, &task)?;

        // If this was the active task, clear the active task record
        if is_active_task {
            active_task::clear_active_task(&self.tasks_dir)?;
            debug!(
                "Completed active task: {} and cleared active task file",
                task_id
            );
        } else {
            debug!("Completed task: {}", task_id);
        }

        Ok(())
    }

    /// Start working on a task
    pub fn start_task(&self, task_id: &str) -> Result<String> {
        // 检查是否已经有活动任务
        if let Some(active) = active_task::load_active_task(&self.tasks_dir)? {
            let active_task_obj = storage::load_task(&self.tasks_dir, &active.task_id)?;
            return Err(anyhow!(
                "There's already an active task: {} - {}. Stop it first.",
                active.task_id,
                active_task_obj.description
            ));
        }

        // 加载任务
        let task = storage::load_task(&self.tasks_dir, task_id)?;

        // 检查任务是否已完成或已中止
        if task.status == TaskStatus::Done {
            return Err(anyhow!("Cannot start a completed task"));
        }
        if task.status == TaskStatus::Aborted {
            return Err(anyhow!("Cannot start an aborted task"));
        }

        // 获取当前时间
        let now = chrono::Utc::now().to_rfc3339();

        // 创建并保存活动任务记录
        let active = ActiveTask::new(task.id.clone(), now);
        active_task::save_active_task(&self.tasks_dir, &active)?;

        debug!("Started task: {} and saved to active task file", task.id);
        Ok(task.id)
    }

    /// Stop working on a task
    pub fn stop_task(&self) -> Result<String> {
        // Check if there's an active task
        let Some(active_task_info) = active_task::load_active_task(&self.tasks_dir)? else {
            // No active task found
            bail!("No active task found. Task might not be in progress.")
        };

        // Load the task
        let mut task = storage::load_task(&self.tasks_dir, &active_task_info.task_id)?;

        // Calculate time spent using the active task record
        let started_time = DateTime::parse_from_rfc3339(&active_task_info.started_at)
            .context("Failed to parse started_at time from active task record")?;
        let now = Utc::now();
        let duration = now.signed_duration_since(started_time.with_timezone(&Utc));

        // Calculate total seconds spent
        let seconds_spent = duration.num_seconds().max(0) as u64;

        // Update task time spent
        task.time_spent = Some(task.time_spent.unwrap_or(0) + seconds_spent);

        // Update task status and timestamps
        task.updated_at = Some(chrono::Utc::now().to_rfc3339());

        // Save the updated task
        storage::save_task(&self.tasks_dir, &task)?;

        // Clear the active task record
        active_task::clear_active_task(&self.tasks_dir)?;

        debug!(
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
                let Some(active_task) = active_task::load_active_task(&self.tasks_dir)? else {
                    bail!("No active task found");
                };
                active_task.task_id
            }
        };
        let mut task = storage::load_task(&self.tasks_dir, &task_id)?;

        // Check if the task is already done or aborted
        if task.status == TaskStatus::Done {
            bail!("Cannot abort a completed task");
        }
        if task.status == TaskStatus::Aborted {
            bail!("Task is already aborted");
        }

        // Check if this is the active task
        let is_active_task = match active_task::load_active_task(&self.tasks_dir)? {
            Some(active) => {
                if active.task_id == *task_id {
                    // Calculate time spent using the active task record
                    let started_time = DateTime::parse_from_rfc3339(&active.started_at)
                        .context("Failed to parse started_at time from active task record")?;
                    let now = Utc::now();
                    let duration = now.signed_duration_since(started_time.with_timezone(&Utc));

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
        task.status = TaskStatus::Aborted;
        task.updated_at = Some(chrono::Utc::now().to_rfc3339());
        task.completed_at = Some(chrono::Utc::now().to_rfc3339());

        // Save the updated task
        storage::save_task(&self.tasks_dir, &task)?;

        // If this was the active task, clear the active task record
        if is_active_task {
            active_task::clear_active_task(&self.tasks_dir)?;
            debug!(
                "Aborted active task: {} and cleared active task file",
                task_id
            );
        } else {
            debug!("Aborted task: {}", task_id);
        }

        Ok(task_id)
    }

    /// Edit task description
    pub fn edit_task_description(&self, task_id: &str) -> Result<String> {
        // Load the task
        let mut task = storage::load_task(&self.tasks_dir, task_id)?;

        // Create a temporary file for editing
        let mut temp_file = tempfile::NamedTempFile::new()?;

        // Write the task description to the temporary file
        temp_file.write_all(task.description.as_bytes())?;

        // Get the editor from environment variable or use a default
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

        // Open the editor
        let status = Command::new(&editor)
            .arg(temp_file.path())
            .status()
            .context(format!("Failed to open editor {}", editor))?;

        if status.success() {
            // Read back the edited content
            let mut temp_file = temp_file.reopen()?;

            // Read the contents of the temporary file
            let mut new_description = String::new();
            temp_file.read_to_string(&mut new_description)?;

            // Trim whitespace
            let new_description = new_description.trim().to_string();

            // Only update if description has changed
            if new_description != task.description {
                task.description = new_description;
                task.updated_at = Some(chrono::Utc::now().to_rfc3339());
                storage::save_task(&self.tasks_dir, &task)?;
            }

            Ok(task.id)
        } else {
            Err(anyhow!("Editor exited with non-zero status"))
        }
    }

    /// Clean tasks based on filters
    pub fn clean_tasks(
        &self,
        priority_filter: Option<Priority>,
        scope_filter: Option<&str>,
        type_filter: Option<String>,
        status_filter: Option<TaskStatus>,
        older_than: Option<u32>,
        force: bool,
        display_manager: &DisplayManager,
    ) -> Result<usize> {
        // Get tasks matching filters
        let tasks = self.list_tasks(
            priority_filter,
            scope_filter,
            type_filter.clone(),
            status_filter,
            None,
            None,
            None,
            false,
        )?;

        // Filter by age if specified
        let tasks = if let Some(days) = older_than {
            let now = Utc::now();
            let cutoff_date = now - Duration::days(days as i64);

            tasks
                .into_iter()
                .filter(|task| {
                    if let Some(completed_at) = &task.completed_at {
                        if let Ok(completed_date) = DateTime::parse_from_rfc3339(completed_at) {
                            return completed_date.with_timezone(&Utc) < cutoff_date;
                        }
                    }
                    false
                })
                .collect()
        } else {
            tasks
        };

        let count = tasks.len();

        // Confirm deletion if not forced
        if count > 0 && !force {
            let message = format!("Are you sure to delete {} tasks?", count);
            if !display_manager.confirm(&message)? {
                return Ok(0);
            }
        }

        // Delete tasks
        for task in tasks {
            storage::delete_task(&self.tasks_dir, &task.id)?;
        }

        Ok(count)
    }

    /// Clone a remote repository
    pub fn clone_repo(&self, url: &str) -> Result<()> {
        GitRepo::clone(&self.tasks_dir, url)?;
        Ok(())
    }

    /// Sync with remote repository
    pub fn sync(&self, prefer: MergeStrategy) -> Result<()> {
        let git_repo = GitRepo::init(&self.tasks_dir)?;
        git_repo.sync(prefer)?;
        Ok(())
    }
}
