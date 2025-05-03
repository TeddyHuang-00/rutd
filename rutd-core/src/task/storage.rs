use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::Result;

use crate::{git::repo::GitRepo, task::model::Task};

/// Save task to TOML file
pub fn save_task(root_dir: &Path, task: &Task) -> Result<()> {
    // Make sure the tasks directory exists
    fs::create_dir_all(root_dir)?;

    // Initialize the Git repository
    let git_repo = GitRepo::init(root_dir)?;

    // Use the task's UUID as the filename
    let file_path = root_dir.join(format!("{}.toml", task.id));

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

/// Locate all potential task files by ID
///
/// The function searches for task files in the specified directory that
/// match the given task ID.
///
/// Short IDs are supported, so if the task ID is "1234", it will match
/// "1234.toml" and "1234-5678.toml", as long as it is enough to uniquely
/// identify the file.
pub fn locate_all_tasks(root_dir: &Path, task_id: &str) -> Result<Vec<PathBuf>> {
    let mut matching_files = Vec::new();

    // Iterate over all TOML files in the directory
    for entry in fs::read_dir(root_dir)? {
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

    Ok(matching_files)
}

/// Locate task file by ID
///
/// This function utilizes the `locate_all_tasks` function to find all potential
/// task files. It will only return a single file if it is unique. If multiple
/// files are found, or if no files are found, an error will be returned.
pub fn locate_task(root_dir: &Path, task_id: &str) -> Result<PathBuf> {
    // Make sure the directory exists
    // Make sure the directory exists
    if !root_dir.exists() {
        anyhow::bail!("Tasks directory does not exist");
    }

    let matching_files = locate_all_tasks(root_dir, task_id)?;

    match matching_files.len() {
        1 => Ok(matching_files[0].to_owned()),
        0 => anyhow::bail!("No task found with ID starting with {}", task_id),
        _ => anyhow::bail!("Multiple tasks found with ID starting with {}", task_id),
    }
}

/// Load task from TOML file
pub fn load_task(root_dir: &Path, task_id: &str) -> Result<Task> {
    let file = locate_task(root_dir, task_id)?;

    // Read the contents of the file
    let mut file = File::open(&file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    // Deserialize the TOML content into a Task struct
    let task = toml::from_str(&contents)?;

    Ok(task)
}

/// Load all tasks
pub fn load_all_tasks(root_dir: &Path) -> Result<Vec<Task>> {
    let mut tasks = Vec::new();

    // Make sure the directory exists
    if !root_dir.exists() {
        return Ok(tasks);
    }

    // Iterate over all TOML files in the directory
    for entry in fs::read_dir(root_dir)? {
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
pub fn delete_task(root_dir: &Path, task_id: &str) -> Result<()> {
    let file = locate_task(root_dir, task_id)?;
    fs::remove_file(file)?;

    // Initialize the Git repository
    let git_repo = GitRepo::init(root_dir)?;

    // Automatically commit changes
    let commit_message = GitRepo::generate_commit_message(task_id, "Delete task");
    git_repo.commit_changes(&commit_message)?;
    Ok(())
}
