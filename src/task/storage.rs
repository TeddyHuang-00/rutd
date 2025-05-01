use crate::git::repo::GitRepo;
use crate::task::model::Task;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use uuid::Uuid;

const TASKS_DIR: &str = ".todos/tasks";

/// Save task to TOML file
pub fn save_task(task: &Task) -> Result<(), Box<dyn std::error::Error>> {
    // Make sure the tasks directory exists
    fs::create_dir_all(TASKS_DIR)?;

    // Initialize the Git repository
    let git_repo = GitRepo::init(".todos")?;

    // Use the task's UUID as the filename
    let file_path = PathBuf::from(TASKS_DIR).join(format!("{}.toml", task.id));

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

/// Load task from TOML file
pub fn load_task(task_id: &str) -> Result<Task, Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(TASKS_DIR).join(format!("{}.toml", task_id));

    // Read the contents of the file
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize the TOML content into a Task struct
    let task: Task = toml::from_str(&contents)?;

    Ok(task)
}

/// Load all tasks
pub fn load_all_tasks() -> Result<Vec<Task>, Box<dyn std::error::Error>> {
    let tasks_dir = PathBuf::from(TASKS_DIR);
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
pub fn delete_task(task_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from(TASKS_DIR).join(format!("{}.toml", task_id));
    if file_path.exists() {
        fs::remove_file(file_path)?;

        // Initialize the Git repository
        let git_repo = GitRepo::init(".todos")?;

        // Automatically commit changes
        let commit_message = GitRepo::generate_commit_message(task_id, "Delete task");
        git_repo.commit_changes(&commit_message)?;
    }
    Ok(())
}
