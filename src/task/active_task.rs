use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};

/// Active Task information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTask {
    /// Task ID
    pub task_id: String,
    /// Time when the task was started
    pub started_at: String,
}

impl ActiveTask {
    /// Create a new active task record
    pub fn new(task_id: String, started_at: String) -> Self {
        Self {
            task_id,
            started_at,
        }
    }
}

/// Save the currently active task
pub fn save_active_task(root_dir: &Path, active_task: &ActiveTask) -> Result<()> {
    let path = root_dir.join("active_task.toml");
    debug!("Saving active task to {}", path.display());

    // Make sure the directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Serialize the active task to TOML
    let toml_string = toml::to_string(active_task)?;

    // Write to file
    let mut file = File::create(&path)?;
    file.write_all(toml_string.as_bytes())?;

    Ok(())
}

/// Load the currently active task, if any
pub fn load_active_task(root_dir: &Path) -> Result<Option<ActiveTask>> {
    let path = root_dir.join("active_task.toml");
    trace!("Checking for active task at {}", path.display());

    if !path.exists() {
        debug!("No active task file found");
        return Ok(None);
    }

    // Read the file
    let mut contents = String::new();
    let mut file = File::open(&path).context(format!(
        "Failed to open active task file at {}",
        path.display()
    ))?;
    file.read_to_string(&mut contents)?;

    // Deserialize from TOML
    let active_task: ActiveTask =
        toml::from_str(&contents).context("Failed to parse active task TOML")?;

    Ok(Some(active_task))
}

/// Clear the active task
pub fn clear_active_task(root_dir: &Path) -> Result<()> {
    let path = root_dir.join("active_task.toml");
    if !path.exists() {
        debug!("No active task file to clear");
        return Ok(());
    }

    // Remove the file
    fs::remove_file(&path)?;

    info!("Cleared active task");
    Ok(())
}
