# Rust Todo Application

A terminal-based todo application built with Rust and Ratatui, featuring workspace management, task tracking, and a user-friendly interface.

## Features

- **Workspace Management**: Create, delete, archive, and organize workspaces to categorize your tasks
- **Task Management**: Add tasks with nested subtasks, track status (todo, in-progress, completed, deprecated)
- **Due Dates**: Set due dates for tasks with flexible date input (specific dates or relative dates like "3 days", "2 weeks")
- **Search & Filter**: Quickly find tasks using search functionality
- **Data Persistence**: Automatically saves your data to a JSON file
- **Keyboard Navigation**: Intuitive keybindings for efficient task management
- **Help System**: Built-in help screen showing all available keybindings

## Installation

### Prerequisites

- Rust and Cargo (latest stable version recommended)

### Building from Source

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd rust-todo
   ```

2. Build the application:
   ```bash
   cargo build --release
   ```

3. Run the application:
   ```bash
   cargo run --release
   ```
   
   Or run the binary directly:
   ```bash
   ./target/release/todo
   ```

## Usage

### Basic Navigation

- **Tab/1/2/3**: Switch between Workspace, Archived Workspace, and Todo List panels
- **Arrow Keys/j/k**: Navigate up and down in lists
- **Enter/l**: Enter a workspace to view its tasks
- **h**: Go back from Todo List to Workspace
- **a**: Add new item (workspace or task depending on focus)
- **i**: Add child item (sub-workspace or sub-task)
- **x**: Delete selected item
- **r**: Rename selected item
- **f/**: Filter/search tasks
- **?**: Show help screen
- **Ctrl+s**: Save data manually
- **q/Esc**: Quit application

### Task Management

- **t**: Mark task as "Todo"
- **p**: Mark task as "In Progress"
- **c**: Mark task as "Completed"
- **d**: Mark task as "Deprecated"
- **D**: Set due date for task

### Workspace Management

- **A**: Archive current workspace
- **R**: Recover archived workspace

### Data Storage

The application automatically saves data to `~/.todo/data.json`. This file contains all your workspaces, tasks, and their statuses.

## Keybindings Reference

| Key | Action |
|-----|--------|
| Tab/1/2/3 | Switch focus between panels |
| j/k or Arrow Keys | Navigate up/down |
| Enter/l | Enter workspace |
| h | Return to workspace from todo list |
| a | Add item |
| i | Add child item |
| x | Delete item |
| r | Rename item |
| f or / | Filter/search |
| ? | Show help |
| Ctrl+s | Save data |
| q or Esc | Quit |

### Task Status Keys

| Key | Status |
|-----|--------|
| t | Todo |
| p | In Progress |
| c | Completed |
| d | Deprecated |
| D | Set Due Date |

### Workspace Keys

| Key | Action |
|-----|--------|
| A | Archive workspace |
| R | Recover workspace |

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.