use std::process::ExitCode;

use rutd_core::{Config, TaskManager};
use rutd_tui::app::TuiApp;

fn main() -> ExitCode {
    // Get configuration from environment variables
    let Ok(config) = Config::new().inspect_err(|e| eprintln!("Failed to load configuration: {e}"))
    else {
        return ExitCode::FAILURE;
    };

    let path_config = config.path;
    let git_config = config.git;

    // Build the task manager
    let task_manager = TaskManager::new(path_config, git_config);

    // Create and run the TUI application
    let app = TuiApp::new(task_manager);
    if let Err(e) = app.run() {
        eprintln!("Error running TUI application: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
