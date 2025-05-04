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

// impl TaskIdCompleter {
//     /// Create a new TaskIdCompleter
//     pub fn new(tasks_dir: String) -> Self {
//         Self { tasks_dir }
//     }
// }

// impl ShellCompleter for TaskIdCompleter {
//     fn generate_completions(
//         &self,
//         _: &Command,
//         _: &ArgMatches,
//         _: &Arg,
//         prefix: &str,
//     ) -> Vec<String> {
//         let task_ids = self.get_task_ids();
//         task_ids
//             .into_iter()
//             .filter(|id| id.starts_with(prefix))
//             .collect()
//     }
// }

// /// Defines a value completer for task scopes
// pub struct TaskScopeCompleter {
//     /// Path to the tasks directory
//     tasks_dir: String,
//     /// Task configuration with pinned scopes
//     task_config: TaskConfig,
// }

// impl TaskScopeCompleter {
//     /// Create a new TaskScopeCompleter
//     pub fn new(tasks_dir: String, task_config: TaskConfig) -> Self {
//         Self {
//             tasks_dir,
//             task_config,
//         }
//     }

//     /// Get a list of task scopes as completion candidates
//     fn get_task_scopes(&self) -> Vec<String> {
//         let mut scopes = self.task_config.scopes.clone();

//         // Add active scopes from existing tasks
//         let path = Path::new(&self.tasks_dir);
//         if path.exists() {
//             if let Ok(tasks) = load_all_tasks(path) {
//                 for task in tasks {
//                     if let Some(scope) = task.scope {
//                         if !scopes.contains(&scope) {
//                             scopes.push(scope);
//                         }
//                     }
//                 }
//             }
//         }

//         scopes
//     }
// }

// impl ShellCompleter for TaskScopeCompleter {
//     fn generate_completions(
//         &self,
//         _: &Command,
//         _: &ArgMatches,
//         _: &Arg,
//         prefix: &str,
//     ) -> Vec<String> {
//         let scopes = self.get_task_scopes();
//         scopes
//             .into_iter()
//             .filter(|scope| scope.starts_with(prefix))
//             .collect()
//     }
// }

// /// Defines a value completer for task types
// pub struct TaskTypeCompleter {
//     /// Path to the tasks directory
//     tasks_dir: String,
//     /// Task configuration with pinned types
//     task_config: TaskConfig,
// }

// impl TaskTypeCompleter {
//     /// Create a new TaskTypeCompleter
//     pub fn new(tasks_dir: String, task_config: TaskConfig) -> Self {
//         Self {
//             tasks_dir,
//             task_config,
//         }
//     }

//     /// Get a list of task types as completion candidates
//     fn get_task_types(&self) -> Vec<String> {
//         let mut types = self.task_config.types.clone();

//         // Add active types from existing tasks
//         let path = Path::new(&self.tasks_dir);
//         if path.exists() {
//             if let Ok(tasks) = load_all_tasks(path) {
//                 for task in tasks {
//                     if let Some(task_type) = task.task_type {
//                         if !types.contains(&task_type) {
//                             types.push(task_type);
//                         }
//                     }
//                 }
//             }
//         }

//         types
//     }
// }

// impl ShellCompleter for TaskTypeCompleter {
//     fn generate_completions(
//         &self,
//         _: &Command,
//         _: &ArgMatches,
//         _: &Arg,
//         prefix: &str,
//     ) -> Vec<String> {
//         let types = self.get_task_types();
//         types
//             .into_iter()
//             .filter(|task_type| task_type.starts_with(prefix))
//             .collect()
//     }
// }

// /// Register completers for all dynamic values in the CLI
// pub fn register_completers(
//     cmd: Command,
//     path_config: &PathConfig,
//     task_config: &TaskConfig,
// ) -> Command {
//     let tasks_dir = path_config.task_dir().to_string_lossy().to_string();

//     // Setup completers
//     let id_completer = TaskIdCompleter::new(tasks_dir.clone());
//     let scope_completer = TaskScopeCompleter::new(tasks_dir.clone(),
// task_config.clone());     let type_completer =
// TaskTypeCompleter::new(tasks_dir, task_config.clone());

//     // Identify all commands with task ID parameters and apply the completer
//     let cmd = register_task_id_completers(cmd, id_completer);

//     // Apply scope and type completers
//     let cmd = register_scope_completers(cmd, scope_completer);
//     let cmd = register_type_completers(cmd, type_completer);

//     cmd
// }

// /// Register task ID completers for all commands that use task IDs
// fn register_task_id_completers(mut cmd: Command, id_completer:
// TaskIdCompleter) -> Command {     // List of subcommands that require task ID
// completions     let id_commands = [
//         "done", "d", "f", "edit", "e", "start", "s", "abort", "x", "c",
//     ];

//     for id_cmd in id_commands {
//         if let Some(subcmd) = cmd.find_subcommand_mut(id_cmd) {
//             subcmd.arg(
//                 Arg::new("id")
//                     .shell_complete_fn(move |cmd, args, arg, prefix| {
//                         id_completer.generate_completions(cmd, args, arg,
// prefix)                     })
//                     .value_hint(ValueHint::Other)
//                     .hide(true), // Hide from help, as it's already defined
// in the command structure             );
//         }
//     }

//     cmd
// }

// /// Register scope completers for the add command
// fn register_scope_completers(mut cmd: Command, scope_completer:
// TaskScopeCompleter) -> Command {     // Apply to the add command
//     if let Some(add_cmd) = cmd.find_subcommand_mut("add") {
//         add_cmd.arg(
//             Arg::new("scope")
//                 .shell_complete_fn(move |cmd, args, arg, prefix| {
//                     scope_completer.generate_completions(cmd, args, arg,
// prefix)                 })
//                 .value_hint(ValueHint::Other)
//                 .hide(true),
//         );
//     }

//     // Also apply to list command's filter
//     if let Some(list_cmd) = cmd.find_subcommand_mut("list") {
//         list_cmd.arg(
//             Arg::new("scope")
//                 .short('c')
//                 .long("scope")
//                 .value_name("scope")
//                 .shell_complete_fn(move |cmd, args, arg, prefix| {
//                     scope_completer.generate_completions(cmd, args, arg,
// prefix)                 })
//                 .value_hint(ValueHint::Other)
//                 .hide(true),
//         );
//     }

//     cmd
// }

// /// Register type completers for the add command
// fn register_type_completers(mut cmd: Command, type_completer:
// TaskTypeCompleter) -> Command {     // Apply to the add command
//     if let Some(add_cmd) = cmd.find_subcommand_mut("add") {
//         add_cmd.arg(
//             Arg::new("task_type")
//                 .shell_complete_fn(move |cmd, args, arg, prefix| {
//                     type_completer.generate_completions(cmd, args, arg,
// prefix)                 })
//                 .value_hint(ValueHint::Other)
//                 .hide(true),
//         );
//     }

//     // Also apply to list command's filter
//     if let Some(list_cmd) = cmd.find_subcommand_mut("list") {
//         list_cmd.arg(
//             Arg::new("task_type")
//                 .short('t')
//                 .long("task-type")
//                 .value_name("type")
//                 .shell_complete_fn(move |cmd, args, arg, prefix| {
//                     type_completer.generate_completions(cmd, args, arg,
// prefix)                 })
//                 .value_hint(ValueHint::Other)
//                 .hide(true),
//         );
//     }

//     cmd
// }
