pub mod model;
pub mod storage;

use crate::task::model::{Priority, Task, TaskStatus, TaskType};
use std::error::Error;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use uuid::Uuid;

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
        task_type: TaskType,
    ) -> Result<String, Box<dyn Error>> {
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
        type_filter: Option<TaskType>,
        status_filter: Option<TaskStatus>,
    ) -> Result<Vec<Task>, Box<dyn Error>> {
        let tasks = storage::load_all_tasks()?;
        let filtered_tasks = tasks.into_iter().filter(|task| {
            Self::matches_filters(
                task,
                &priority_filter,
                &scope_filter,
                &type_filter,
                &status_filter,
            )
        }).collect();
        Ok(filtered_tasks)
    }

    /// Check if a task matches the filter conditions
    fn matches_filters(
        task: &Task,
        priority_filter: &Option<Priority>,
        scope_filter: &Option<&str>,
        type_filter: &Option<TaskType>,
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
        if let Some(t) = type_filter {
            if task.task_type != *t {
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
    pub fn mark_task_done(&self, task_id: &str) -> Result<(), Box<dyn Error>> {
        let mut task = storage::load_task(task_id)?;
        task.status = TaskStatus::Done;
        task.updated_at = Some(chrono::Utc::now().to_rfc3339());
        task.completed_at = Some(chrono::Utc::now().to_rfc3339());
        storage::save_task(&task)?;
        Ok(())
    }

    /// Edit task description
    pub fn edit_task_description(&self, task_id: &str) -> Result<(), Box<dyn Error>> {
        let task_path = Path::new(".todos/tasks").join(format!("{}.toml", task_id));
        if task_path.exists() {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
            let status = Command::new(&editor)
                .arg(&task_path)
                .status()?;
            if status.success() {
                let mut task = storage::load_task(task_id)?;
                task.updated_at = Some(chrono::Utc::now().to_rfc3339());
                storage::save_task(&task)?;
                Ok(())
            } else {
                Err(format!("编辑器 {} 未能成功编辑任务", editor).into())
            }
        } else {
            Err(format!("任务 {} 未找到", task_id).into())
        }
    }
}