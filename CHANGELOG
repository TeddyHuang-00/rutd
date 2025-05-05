# Changelog

All notable changes to this project will be documented in this file.

## [0.6.0] - 2025-05-05

### 🚀 Features

- Add aliases for priority and task status enums
- Add strum for better interoperability between core and cli
- *(core)* Add sorting functionality for tasks with customizable sort options
- *(core)* Enhance MergeStrategy and Task enums with strum for better serialization and display
- *(cli)* Enhance CLI with sorting and smarter auto-completion

### 🐛 Bug Fixes

- *(cli)* Format completed_at timestamp for better readability

### 📚 Documentation

- Update README for improved clarity and consistency in CLI commands and features
- *(cli)* Add new demonstration vhs tapes
- Add comprehensive usage guide with visual demonstrations

### 🧪 Testing

- *(cli)* Add comprehensive tests for display, filter, completer and parser modules
- *(core)* Refactor tests to use shared task creation function; enhance display mock; use env guard for test environment
- *(cli)* Refactor tests to remove MockConfig; enhance environment setup and cleanup; add tests for completion functions

### ⚙️ Miscellaneous Tasks

- *(tools)* Add coverage report
- Update .gitignore to include  coverage reports
- Update tag detection logic to also include stable versions for pre-release
- Add demonstration recipes for visual demonstration
- Include USAGE.md in version control
- Add default configuration file for git-cliff

## [0.5.0] - 2025-05-05

### 🚜 Refactor

- Move completion logic to cli crate for clarity
- Simplify conflict resolution path handling in merge strategy

### 📚 Documentation

- Update README to enhance project description and installation instructions
- Update installation instructions for clarity and completeness
- Update shell completion instructions for CLI and add support for Windows

### ⚙️ Miscellaneous Tasks

- Add GitHub Actions workflow for releasing binaries and updating changelog
- Fix failing release workflow
- Add Windows target configurations for testing
- Add Windows target configurations and improve artifact handling in release workflow
- Enhance release workflow to handle pre-release versions and update changelog accordingly
- Improve pre-release detection logic and add logging for previous tags
- Update pre-release detection logic to use --no-pager for git tags
- Update release workflow to improve version detection and output variable names
- Update version determination logic to fetch all history and tags

## [0.4.0] - 2025-05-04

### 🚀 Features

- *(ci)* Add GitHub Actions workflow for testing build binaries
- *(core)* Enhance relative date parsing to support multiple units
- *(cli)* Rename argument and value placeholder for better semantic
- Add vendored features for OpenSSL and Git2 dependencies

### 🐛 Bug Fixes

- *(core)* Enhance task storage functions with improved commit message formatting
- *(ci)* Add armv7 target for Linux and install cross-compilation tools
- *(ci)* Update armv7 target and install additional dependencies for cross-compilation
- *(ci)* Install pkg-config for cross-compilation tools
- *(ci)* Update Ubuntu version in build matrix and improve verbosity of Cargo build
- *(ci)* Add feature specification for build targets and remove OpenSSL configuration step
- *(ci)* Add zlib feature for armv7 target in test workflow
- Update dependencies in Cargo.toml files for consistency
- *(ci)* Remove armv7 target and cross-compilation tools installation step from test workflow
- Remove unused zlib feature from Cargo.toml files
- *(ci)* Restore installation of cross-compilation tools in test workflow
- Ensure code formatting is also applied after clippy checks
- Update clippy command and add clippy lints configuration

### 🚜 Refactor

- *(ci)* Simplify build matrix by removing cross compilation targets
- Refactor task filtering and CLI integration
- Refactor parsing from core to cli.

### 🧪 Testing

- Add unit testing and fix bugs

### ⚙️ Miscellaneous Tasks

- Add Renovate configuration file for automated dependency management
- Add GitHub Actions workflow for code formatting, Clippy checks, and unit testing
- Add missing components for rustfmt and clippy in workflow
- Rename workflow from "Test Build Binaries" to "Test"

## [0.3.0] - 2025-05-04

### 🚀 Features

- Add command completion support for CLI
- Add command completion support for CLI and restructure main entry points
- *(core)* Refactor code for better maintenance
- *(core)* Add TaskConfig structure for task management and autocompletion
- *(core)* Add TaskStatus to task module re-exports
- *(core)* Add command-line completion for task IDs, scopes, and types
- *(cli)* Add command-line completion for task ID, scope, and type arguments

### 🐛 Bug Fixes

- Update logging level configuration in init_logger function
- *(cli)* Fix task module imports
- *(tui)* Allow dead code for future TUI implementation
- *(cli)* Update edit_task_description method to use DisplayManager

### 🚜 Refactor

- Simplify string formatting in logging and display modules
- Streamline release process by removing confirmation prompt
- Extract package name resolution logic in Config::new function
- *(core)* Update config file path formatting in Config::new function
- *(core)* Remove tempfile dependency and update edit_task_description method
- Replace log macros with log:: prefix in various modules

### 📚 Documentation

- Add shell completion instructions for various shells in README

### ⚙️ Miscellaneous Tasks

- Add justfile for release management and code formatting

## [0.2.1] - 2025-05-03

### 🚀 Features

- Initialize RuTD project with basic CLI for task management
- Update dependencies in Cargo.toml and Cargo.lock
- Add rustfmt configuration file for code formatting
- Enhance CLI commands with verbosity option and improve task management features
- Add new dependencies for improved functionality in Cargo.toml and Cargo.lock
- Add MIT License file to the repository
- Add filtering options for task listing and enhance task management commands
- Enhance Git synchronization with improved authentication and error handling
- Implement active task management with save, load, and clear functionalities
- Add time spent field to Task struct for better tracking
- Enhance task management with filtering, statistics, and active task handling
- Enhance task listing with additional filtering options and statistics
- Add MergeStrategy enum for customizable merge behavior
- Implement repository cloning and enhance sync with customizable merge strategy
- Add repository cloning functionality and update sync method to accept MergeStrategy
- Add abbreviation for value enums
- Update task commands to support stopping and aborting active tasks without specifying IDs
- Add visible aliases for command subcommands in CLI
- Add comfy-table dependency for improved table formatting
- Implement DisplayManager for user interface output handling
- Enhance path configuration management and refactor task handling
- Add figment dependency for enhanced configuration management
- Add Git and Path configuration management for enhanced task handling
- Update Git and Path configuration to enforce non-optional fields and improve serialization
- Enhance success and failure message display in CLI mode with colored output
- Replace failure messages with trace logs for task commands
- Implement logging configuration with file output support
- Refactor logging and task management to support configurable log file paths and history limits
- Refactor task filtering to use a dedicated FilterOptions struct for improved clarity and maintainability
- Refactor task filtering and logging to use local time and improve date handling
- Enhance task filtering with creation, update, and completion date ranges
- Add CLI feature support for clap with value enums and filter options
- Add rutd-cli as a dependency and refactor main application logic

### 🐛 Bug Fixes

- Change default editor from "vi" to "nano" and trim task description on edit
- Make verbosity level argument global in CLI
- Update dependencies for rutd to include rutd-cli and rutd-tui
- Add version specification for rutd-core dependency in Cargo.toml
- Update rutd-core dependency version to 0.2.1-rc.1 in Cargo.toml files
- Update rutd, rutd-cli, and rutd-tui dependencies to version 0.2.1-rc.1 in Cargo.toml and Cargo.lock
- Remove rutd-cli and rutd-tui dependencies from Cargo.toml and Cargo.lock
- Add readme specification in Cargo.toml for rutd-cli, rutd-core, and rutd-tui

### 🚜 Refactor

- Update comments in test module to English
- Simplify repository initialization logic in GitRepo
- Update task storage path to use home directory and improve task file handling
- Simplify commit creation logic by consolidating parent handling
- Refactor task management and storage functions for improved clarity and structure
- Reorganize imports for clarity and consistency across modules
- Update comments to improve clarity and consistency in task management
- Move general config to log config
- Remove unused Duration import from chrono
- Update FilterOptions to use optional date range for improved flexibility
- Refactor core functionality for task management and logging separate from apps

### 📚 Documentation

- Update README with roadmap
- Update comments to clarify visible aliases support for value enums in clap
- Enhance date range filtering documentation with detailed format specifications

### ⚙️ Miscellaneous Tasks

- Add .todos to .gitignore to exclude todo files from version control
- Remove .gitkeep file from .todos/tasks directory
- Update .gitignore to ignore all markdown files except README.md
- Update dependencies and change edition to 2024
- Update package metadata in Cargo.toml
- Update categories in Cargo.toml for better classification
- Update cc package version to 1.2.21 and checksum in Cargo.lock
- Remove unused ValueEnum import from commands.rs and clean up repo.rs
- Update dependencies in Cargo.toml and Cargo.lock for improved functionality
- Update dependencies for improved performance and stability
- Update package versions to 0.2.0 in Cargo.lock and Cargo.toml

<!-- generated by git-cliff -->
