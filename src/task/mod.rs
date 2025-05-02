pub mod model;
pub mod storage;

use std::{error::Error, fs, io::Write, path::Path, process::Command};

use anyhow::{Result, anyhow};
use uuid::Uuid;

use crate::task::model::{Priority, Task, TaskStatus};

/// Task Manager
pub struct TaskManager {
    tasks_dir: String,
}

impl TaskManager {
    /// Create a new Task Manager
    pub fn new(tasks_dir: &str) -> Self {
        TaskManager {
            tasks_dir: tasks_dir.to_string(),
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
        storage::save_task(&task)?;
        Ok(id)
    }

    /// List tasks with filtering support
    pub fn list_tasks(
        &self,
        priority_filter: Option<Priority>,
        scope_filter: Option<&str>,
        type_filter: Option<String>,
        status_filter: Option<TaskStatus>,
    ) -> Result<Vec<Task>> {
        let tasks = storage::load_all_tasks()?;
        let filtered_tasks = tasks
            .into_iter()
            .filter(|task| {
                Self::matches_filters(
                    task,
                    &priority_filter,
                    &scope_filter,
                    &type_filter,
                    &status_filter,
                )
            })
            .collect();
        Ok(filtered_tasks)
    }

    /// Check if a task matches the filter conditions
    fn matches_filters(
        task: &Task,
        priority_filter: &Option<Priority>,
        scope_filter: &Option<&str>,
        type_filter: &Option<String>,
        status_filter: &Option<TaskStatus>,
    ) -> bool {
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
        true
    }

    /// Mark a task as completed
    pub fn mark_task_done(&self, task_id: &str) -> Result<()> {
        let mut task = storage::load_task(task_id)?;
        task.status = TaskStatus::Done;
        task.updated_at = Some(chrono::Utc::now().to_rfc3339());
        task.completed_at = Some(chrono::Utc::now().to_rfc3339());
        storage::save_task(&task)?;
        Ok(())
    }

    /// Edit task description
    pub fn edit_task_description(&self, task_id: &str) -> Result<()> {
        let task_path = Path::new(".todos/tasks").join(format!("{}.toml", task_id));
        if task_path.exists() {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let status = Command::new(&editor).arg(&task_path).status()?;
            if status.success() {
                let mut task = storage::load_task(task_id)?;
                task.updated_at = Some(chrono::Utc::now().to_rfc3339());
                storage::save_task(&task)?;
                Ok(())
            } else {
                Err(anyhow!("{} fails to edit the task file", editor))
            }
        } else {
            Err(anyhow!("Task {} not found", task_id))
        }
    }
}
