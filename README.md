# RuTD

RuTD stands for "Rush To Do" or "Rust To Do". It is a simple command line tool for managing your to-do list in Rust. It is designed to be fast, easy to use, flexible, and git friendly.

⚠️ RuTD is a work in progress and is not yet feature complete.

## Shell completion

RuTD supports shell completion with `clap_complete`. For better experience, RuTD uses dynamic completion for commands, so it is recommended to source the completion script in your shell configuration file. Specific instructions for each shell are provided below.

### Bash

Add the following line to your `~/.bashrc` or `~/.bash_profile`:

```bash
source <(COMPLETE=bash your_program)
```

### Zsh

Add the following line to your `~/.zshrc`:

```zsh
source <(COMPLETE=zsh your_program)
```

### Fish

Add the following line to your `~/.config/fish/config.fish`:

```fish
source (COMPLETE=fish your_program | psub)
```

### Elvish

Add the following line to your `~/.elvish/rc.elv`:

```elvish
eval (E:COMPLETE=elvish your_program | slurp)
```

### Xonsh

Install `fish` shell and follow the steps for [`fish` completion](#fish). Then, install `xonsh-fish-completer` and add the following line to your `~/.xonshrc`:

```xsh
xontrib load fish_completer
```

## Roadmap

Below is the development roadmap for the RuTD project, based on the planned phased feature rollout:

### Phase 1: MVP - Core Features and Git Integration **(Completed)**

- **Core Task Management (CLI):**
  - Add tasks, including description, priority (Urgent, High, Normal, Low), scope (`<project-name>`), and type (`<feat|fix|docs|etc.>`).
  - List tasks, supporting basic filtering (priority, scope, type, status).
  - Mark tasks as completed.
  - Edit task descriptions via `$EDITOR`.
- **Storage:**
  - Implement TOML-based storage, one file per task, using UUID as filename, stored in the `~/.rutd` directory.
  - Use Serde for serialization/deserialization.
- **Basic Git Integration:**
  - Initialize a Git repository in `~/.rutd` if one does not exist.
  - Automatically commit changes to task files (add, edit, complete).
  - Generate basic commit messages (e.g., `chore(<task_id>): Update task`).
- **Basic CLI:**
  - Use `clap` to implement core commands (`add`, `list`, `done`, `edit`).
  - Generate basic help information.

### Phase 2: Enhanced Task Management and Git Workflow **(Work in Progress)**

- **Advanced Task Management (CLI):**
  - Implement task state transitions: `start`, `stop`, `abort`.
  - Implement time tracking related to `start`/`stop` actions (`time_spent`), using `chrono`.
  - Enforce "single active task" rule.
  - Enhance `list` command: filter by completion date range, fuzzy description matching (using `fuzzy-matcher`), display statistics (count, total time spent).
  - Implement `clean` command to delete tasks based on filters.
- **Improved Git Integration:**
  - Automatically generate Conventional Commit messages based on action type and task details (scope, ID), e.g., `<type>(<scope>): Mark task as done\n\n<id>`.
  - Implement manual `sync` command (fetch/pull + push), using `git2-rs`.
  - Basic push/pull authentication handling (default SSH key path, username/password via environment variable or prompt).
- **CLI Enhancements:**
  - Use `clap_complete` to implement static shell completion (e.g., priority, status).

### Phase 3: TUI, Background Sync, and Optimization **(Not Completed)**

- **Terminal User Interface (TUI):**
  - Develop TUI using `Ratatui` and `crossterm`.
  - Display tasks in interactive lists/tables.
  - Allow task selection, view details, and trigger actions via shortcuts (start/stop/finish/edit).
  - Integrate launching `$EDITOR` for editing from TUI.
- **Background Sync:**
  - Use `notify` to implement file system monitoring, detecting changes in the task directory.
  - Trigger automatic `sync` operations (commit, pull, push) on change detection, with debounce handling.
  - Use `service-manager` for cross-platform background service management (install/start/stop).
- **Advanced Git and Optimization:**
  - Implement application-aware merge conflict resolution strategies (detect conflicts, deserialize versions, prompt user via CLI/TUI for field-level resolution).
  - Implement `gc` command by invoking `git gc` (possibly using `--auto` or configurable `--prune`) for repository optimization.
  - Enhanced authentication (SSH agent, credential helpers via `auth-git2-rs` or similar).
- **CLI/TUI Enhancements:**
  - Use `clap_complete` and custom completers to implement dynamic shell completion (e.g., existing scopes, types, and task IDs).

### Future Considerations (Post-Phase 3) **(Not Completed)**

- **Schema Validation:** Integrate schema validation (e.g., use `schemars` for generation and Serde deserialization for runtime checks).
- **Advanced TUI Editing:** Implement in-TUI editing features, not just relying on `$EDITOR`.
- **Reporting and Visualization:** Add more advanced reporting features or TUI visualizations (e.g., calendar view, progress summary).
- **Plugin System:** Explore a WASI-based plugin system for extensibility.
- **Alternative Git Backend:** If `gitoxide` matures and its feature set (especially around native `gc`) becomes advantageous, evaluate migrating from `git2-rs` to `gitoxide`.
