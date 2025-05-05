# RuTD - Rust Todo Manager

[![MIT License](https://img.shields.io/github/license/TeddyHuang-00/rutd)](./LICENSE)
[![Crates.io Version](https://img.shields.io/crates/v/rutd)](https://crates.io/crates/rutd)
[![Crates.io Total Downloads](https://img.shields.io/crates/d/rutd)](https://crates.io/crates/rutd)
[![GitHub branch check runs](https://img.shields.io/github/check-runs/TeddyHuang-00/rutd/main?cacheSeconds=60)](https://github.com/TeddyHuang-00/rutd/actions)
[![GitHub Stars](https://img.shields.io/github/stars/TeddyHuang-00/rutd)](https://github.com/TeddyHuang-00/rutd)

RuTD ("Rust To Do" or "Rush To Do") is a high-performance todo list manager built in Rust. It's designed for developers and power users who value efficiency, control, and Git integration in their task management workflow.

**Status**: RuTD is stable and feature-rich with core and advanced functionality complete. New features are being actively developed.

![Demo](https://raw.githubusercontent.com/TeddyHuang-00/rutd/main/assets/gif/start-stop.gif)

## Key Features

- **Fast & Lightweight**: Written in Rust for excellent performance and minimal resource usage
- **Git-Integrated**: Automatically tracks task changes in Git for version control and syncing
- **CLI-First**: Powerful command-line interface with intuitive commands and filtering
- **Dynamic Shell Completions**: Built-in completion for Bash, Zsh, Fish, and Elvish
- **Advanced Filtering**: Powerful query capabilities for finding and organizing tasks
- **Flexible Storage**: Task data stored in TOML format, one file per task
- **Developer-Friendly**: Designed by developers, for developers
- **Conventional Commits**: Automated commit message generation following standards

## Installation

### Pre-built Binaries

Pre-built binaries are available for Linux, macOS and Windows. You can download the latest release from the [Releases page](https://github.com/TeddyHuang-00/rutd/releases).

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

Here are some quick commands to get you started with RuTD:

```bash
# Add a new task
rutd-cli add "Implement new feature" --priority high --scope backend --type feat

# List all tasks
rutd-cli list

# List high priority tasks
rutd-cli list --priority high

# Mark a task as done (replace task-id with the actual ID)
rutd-cli done task-id

# Edit a task
rutd-cli edit task-id
```

See the [Usage](https://github.com/TeddyHuang-00/rutd/blob/main/USAGE.md) for more detailed commands and options.

## Shell Completion

RuTD supports shell completion with `clap_complete`. For better experience, RuTD uses dynamic completion for commands, so it is recommended to source the completion script in your shell configuration file.

### Bash

Run the following command in your terminal to add the completion script to your `~/.bashrc`:

```bash
echo "source <(COMPLETE=bash rutd-cli)" >> ~/.bashrc
```

### Zsh

Run the following command in your terminal to add the completion script to your `~/.zshrc`:

```zsh
echo "source <(COMPLETE=zsh rutd-cli)" >> ~/.zshrc
```

### Fish

Run the following command in your terminal to add the completion script to your `~/.config/fish/config.fish`:

```fish
echo "source (COMPLETE=fish rutd-cli | psub)" >> ~/.config/fish/config.fish
```

### Elvish

Run the following command in your terminal to add the completion script to your `~/.elvish/rc.elv`:

```elvish
echo "eval (E:COMPLETE=elvish rutd-cli | slurp)" >> ~/.elvish/rc.elv
```

### Xonsh

Install `fish` shell and follow the steps for [`fish` completion](#fish). Then, install `xontrib-fish-completer` and add the following line to your `~/.xonshrc`:

```xonsh
xontrib load fish_completer
```

### PowerShell

Run the following command in PowerShell to add the completion script to your profile:

```powershell
$env:COMPLETE = "powershell"
echo "rutd-cli | Out-String | Invoke-Expression" >> $PROFILE
Remove-Item Env:\COMPLETE
```

## Current Status & Roadmap

RuTD development follows a phased approach:

- **✅ Completed Phases**: Core task management, Git integration, task state transitions, time tracking, filtering, and shell completions are all implemented and stable.

- **🔄 Current Phase (In Progress)**:
  - [x] Dynamic shell completions
  - [x] Windows support
  - [x] Custom sorting
  - [ ] Time-based conflict resolution
  - [ ] Background synchronization
  - [ ] Configuration command (export default, edit, etc.)
  - [ ] Terminal User Interface (TUI) development
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

This project is licensed under the MIT License - see the [LICENSE](https://github.com/TeddyHuang-00/rutd/blob/main/LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Change Log

See the [CHANGELOG](https://github.com/TeddyHuang-00/rutd/blob/main/CHANGELOG.md) for a detailed list of changes and updates.
