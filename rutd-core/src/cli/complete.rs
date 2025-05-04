use std::{collections::HashSet, ffi::OsStr, path::Path};

use clap_complete::CompletionCandidate;

use crate::{
    Config,
    task::{Task, storage},
};

/// Get all tasks from the task directory with error handling
fn get_tasks(task_dir: &Path) -> Vec<Task> {
    if !task_dir.exists() {
        return vec![];
    }

    storage::load_all_tasks(task_dir).unwrap_or_default()
}

/// Get a list of task IDs as completion candidates
pub fn complete_id(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let Config { path, .. } = Config::new().unwrap();
    let tasks = get_tasks(&path.task_dir());
    tasks
        .into_iter()
        // Get the task IDs
        .map(|task| task.id)
        // Keep only those that start with the current prefix
        .filter(|id| id.starts_with(current))
        // Remove duplicates
        .collect::<HashSet<_>>()
        .into_iter()
        // Convert to completion candidates
        .map(CompletionCandidate::new)
        .collect()
}

/// Get a list of scopes as completion candidates
pub fn complete_scope(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let Config { path, task, .. } = Config::new().unwrap();
    let tasks = get_tasks(&path.task_dir());
    // Get the scopes from the task configuration
    task.scopes
        .into_iter()
        // Get the scopes from the tasks
        .chain(tasks.iter().filter_map(|task| task.scope.clone()))
        // Keep only those that start with the current prefix
        .filter(|scope| scope.starts_with(current))
        // Remove duplicates
        .collect::<HashSet<_>>()
        .into_iter()
        // Convert to completion candidates
        .map(CompletionCandidate::new)
        .collect()
}

/// Get a list of task types as completion candidates
pub fn complete_type(current: &OsStr) -> Vec<CompletionCandidate> {
    let Some(current) = current.to_str() else {
        return vec![];
    };

    let Config { path, task, .. } = Config::new().unwrap();
    let tasks = get_tasks(&path.task_dir());
    // Get the task types from the task configuration
    task.types
        .into_iter()
        // Get the task types from the tasks
        .chain(tasks.iter().filter_map(|task| task.task_type.clone()))
        // Keep only those that start with the current prefix
        .filter(|task_type| task_type.starts_with(current))
        // Remove duplicates
        .collect::<HashSet<_>>()
        .into_iter()
        // Convert to completion candidates
        .map(CompletionCandidate::new)
        .collect()
}
