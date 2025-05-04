# RuTD - Rust Todo Manager

[![MIT License](https://img.shields.io/github/license/TeddyHuang-00/rutd)](./LICENSE)
[![Crates.io Version](https://img.shields.io/crates/v/rutd)](https://crates.io/crates/rutd)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/rutd)](https://crates.io/crates/rutd)
[![GitHub branch status](https://img.shields.io/github/checks-status/TeddyHuang-00/rutd/main)](https://github.com/TeddyHuang-00/rutd/actions)
[![GitHub Stars](https://img.shields.io/github/stars/TeddyHuang-00/rutd)](https://github.com/TeddyHuang-00/rutd)

RuTD ("Rust To Do" or "Rush To Do") is a high-performance todo list manager built in Rust. It's designed for developers and power users who value efficiency, control, and Git integration in their task management workflow.

**Status**: RuTD is stable and feature-rich with core and advanced functionality complete. New features are being actively developed.

## Key Features

- **Fast & Lightweight**: Written in Rust for excellent performance and minimal resource usage
- **Git-Integrated**: Automatically tracks task changes in Git for version control and syncing
- **CLI-First**: Powerful command-line interface with intuitive commands and filtering
- **Flexible Storage**: Task data stored in TOML format, one file per task
- **Developer-Friendly**: Designed by developers, for developers
- **Conventional Commits**: Automated commit message generation following standards
- **Dynamic Shell Completions**: Built-in completion for Bash, Zsh, Fish, and Elvish
- **Advanced Filtering**: Powerful query capabilities for finding and organizing tasks

## Installation

### Pre-built Binaries

Pre-built binaries are available for Linux, macOS. You can download the latest release from the [Releases page](https://github.com/TeddyHuang-00/rutd/releases).

### Build from source

RuTD provides both command-line and TUI interfaces (coming soon). You can install both via:

```bash
cargo install rutd
```

or you can install only the CLI version (and TUI will work the same way in the future):

```bash
cargo install rutd-cli
```

## Quick Start

```bash
# Add a new task
rutd add "Implement new feature" --priority high --scope backend --type feat

# List all tasks
rutd list

# List high priority tasks
rutd list --priority high

# Mark a task as done (replace task-id with the actual ID)
rutd done task-id

# Edit a task
rutd edit task-id
```

## Shell Completion

RuTD supports shell completion with `clap_complete`. For better experience, RuTD uses dynamic completion for commands, so it is recommended to source the completion script in your shell configuration file.

### Bash

```bash
source <(COMPLETE=bash rutd)
```

### Zsh

```zsh
source <(COMPLETE=zsh rutd)
```

### Fish

```fish
source (COMPLETE=fish rutd | psub)
```

### Elvish

```elvish
eval (E:COMPLETE=elvish rutd | slurp)
```

## Current Status & Roadmap

RuTD development follows a phased approach:

- **✅ Completed Phases**: Core task management, Git integration, task state transitions, time tracking, filtering, and shell completions are all implemented and stable.

- **🔄 Current Phase (In Progress)**:
  - [x] Dynamic shell completions
  - [ ] Time-based conflict resolution
  - [ ] Background synchronization
  - [ ] Custom sorting
  - [ ] Configuration command (export default, edit, etc.)
  - [ ] Terminal User Interface (TUI) development
  - [ ] Windows support
- **🔮 Future Enhancements (Planned)**:
  - Configuration schema validation
  - Advanced TUI editing
  - Reporting and visualization
  - Plugin system
  - Alternative Git backend options

## Configuration

Tasks are stored in `~/.rutd` directory by default. Configuration options will be expanded in upcoming releases.

## Acknowledgments

RuTD draws inspiration from:

- [dstask](https://github.com/naggie/dstask) for its CLI-first approach to task management
- [taskwarrior](https://taskwarrior.org/) for advanced task filtering concepts

And the development process heavily relies on LLM tools. Huge shout out to:

- [Roo Code](https://roocode.com/)
- [GitHub Copilot](https://github.com/features/copilot)
- [Claude AI](https://claude.ai/)
- [xAI](https://x.ai/)
- [Qwen](https://qwenlm.github.io/)

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
