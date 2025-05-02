use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use shellexpand::tilde;
use uuid::Uuid;

use crate::{git::repo::GitRepo, task::model::Task};

const TASKS_DIR: &str = "~/.rutd";

/// Save task to TOML file
pub fn save_task(task: &Task) -> Result<()> {
    // Expand the tilde in the tasks directory path
    let tasks_dir = tilde(TASKS_DIR).to_string();

    // Make sure the tasks directory exists
    fs::create_dir_all(&tasks_dir)?;

    // Initialize the Git repository
    let git_repo = GitRepo::init(&tasks_dir)?;

    // Use the task's UUID as the filename
    let file_path = PathBuf::from(&tasks_dir).join(format!("{}.toml", task.id));

    // Serialize the task to TOML format
    let toml_string = toml::to_string(task)?;

    // Write the serialized TOML string to a file
    let mut file = File::create(file_path)?;
    file.write_all(toml_string.as_bytes())?;

    // Automatically commit changes
    let commit_message = GitRepo::generate_commit_message(&task.id.to_string(), "Update task");
    git_repo.commit_changes(&commit_message)?;

    Ok(())
}

/// Locate task file by ID
///
/// The function searches for a task file in the specified directory that
/// matches the given task ID.
///
/// Short IDs are supported, so if the task ID is "1234", it will match
/// "1234.toml" and "1234-5678.toml", as long as it is enough to uniquely
/// identify the file.
pub fn locate_task(task_id: &str) -> Result<PathBuf> {
    let tasks_dir = PathBuf::from(tilde(TASKS_DIR).to_string());
    // Make sure the directory exists
    if !tasks_dir.exists() {
        anyhow::bail!("Tasks directory does not exist");
    }
    let mut matching_files = Vec::new();

    // Iterate over all TOML files in the directory
    for entry in fs::read_dir(tasks_dir)? {
        let path = entry?.path();

        if path.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("toml")
            && path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|stem| stem.starts_with(task_id))
        {
            matching_files.push(path);
        }
    }

    match matching_files.len() {
        1 => Ok(matching_files.pop().unwrap()),
        0 => anyhow::bail!("No task found with ID starting with {}", task_id),
        _ => anyhow::bail!("Multiple tasks found with ID starting with {}", task_id),
    }
}

/// Load task from TOML file
pub fn load_task(task_id: &str) -> Result<Task> {
    let file = locate_task(task_id)?;

    // Read the contents of the file
    let mut file = File::open(&file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize the TOML content into a Task struct
    let task = toml::from_str(&contents)?;

    Ok(task)
}

/// Load all tasks
pub fn load_all_tasks() -> Result<Vec<Task>> {
    let tasks_dir = PathBuf::from(tilde(TASKS_DIR).to_string());
    let mut tasks = Vec::new();

    // Make sure the directory exists
    if !tasks_dir.exists() {
        return Ok(tasks);
    }

    // Iterate over all TOML files in the directory
    for entry in fs::read_dir(tasks_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
            let mut contents = String::new();
            let mut file = File::open(&path)?;
            file.read_to_string(&mut contents)?;

            if let Ok(task) = toml::from_str(&contents) {
                tasks.push(task);
            }
        }
    }

    Ok(tasks)
}

/// Delete task file
pub fn delete_task(task_id: &str) -> Result<()> {
    let file = locate_task(task_id)?;
    fs::remove_file(file)?;

    // Initialize the Git repository
    let git_repo = GitRepo::init(tilde(TASKS_DIR).to_string())?;

    // Automatically commit changes
    let commit_message = GitRepo::generate_commit_message(task_id, "Delete task");
    git_repo.commit_changes(&commit_message)?;
    Ok(())
}
