# RuTD Usage Guide

This guide provides detailed instructions on how to use RuTD (Rust Todo Manager), complete with visual demonstrations.

## Table of Contents

- [Table of Contents](#table-of-contents)
- [Getting Help](#getting-help)
- [Task Management](#task-management)
  - [Adding Tasks](#adding-tasks)
  - [Listing and Filtering Tasks](#listing-and-filtering-tasks)
  - [Starting and Stopping Tasks](#starting-and-stopping-tasks)
  - [Completing Tasks](#completing-tasks)
- [Git Integration](#git-integration)
  - [Cloning and Syncing](#cloning-and-syncing)
- [Advanced Usage](#advanced-usage)

## Getting Help

RuTD offers comprehensive help information for all commands. Use the `--help` flag to access it.

![Help Command Demo](assets/gif/help.gif)

## Task Management

### Adding Tasks

Add new tasks to your list with the `add` command. You can specify various attributes like priority, type, or scope.

```bash
rutd-cli add "Implement new feature" --priority high --scope backend --type feat
```

![Adding Tasks Demo](assets/gif/add.gif)

### Listing and Filtering Tasks

List all your tasks with the `list` command. Apply filters to narrow down results.

```bash
# List all tasks
rutd-cli list

# List high priority tasks
rutd-cli list --priority high

# List tasks with specific scope
rutd-cli list --scope backend
```

![Filtering Tasks Demo](assets/gif/filter.gif)

You can also sort the tasks with any order of attributes, we provide advanced dynamic autocompletion for composing your sorting.

```bash
# List tasks sorted by priority (descending) and scope (ascending)
rutd-cli list --sort -p+s
```

![Autocomplete Demo](assets/gif/autocomplete.gif)

### Starting and Stopping Tasks

Track time spent on tasks with start and stop commands:

```bash
# Start working on a task
rutd-cli start <task-id>

# Stop the current active task
rutd-cli stop
```

![Start and Stop Demo](assets/gif/start-stop.gif)

### Completing Tasks

Mark tasks as done or abort them if things don't go as planned.

```bash
# Mark a task as done
rutd-cli done <task-id>

# Abort a task
rutd-cli abort <task-id>
```

![Done and Abort Demo](assets/gif/done-abort.gif)


## Git Integration

### Cloning and Syncing

RuTD integrates with Git for version control and synchronization.

```bash
# Clone tasks from a repository for the first time
rutd-cli clone <repository-url>

# Sync changes with remote repository
rutd-cli sync
```

![Clone and Sync Demo](assets/gif/clone-sync.gif)

## Advanced Usage

For more advanced usage, please use the `--help` flag with specific commands.

```bash
rutd-cli <command> --help
```

Remember that RuTD provides dynamic shell completions which can greatly enhance your experience. See the [README](README.md) for instructions on setting up completions for your shell.
