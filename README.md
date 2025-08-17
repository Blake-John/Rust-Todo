# Rust Todo Application

A terminal-based todo application built with Rust and Ratatui, featuring workspace management, task tracking, and a user-friendly interface.

## Features

<https://github.com/user-attachments/assets/e2796daa-c7c9-499c-86c0-a8fa1a4ce63a>

- **Workspace Management**: Create, delete, archive, and organize workspaces to categorize your tasks, supporting nested sub-workspaces
- **Task Management**: Add tasks with nested subtasks and track their status (`todo`, `in-progress`, `completed`, `deprecated`)
- **Due Dates**: Set due dates for tasks with flexible date input (specific dates or relative dates like `3 days`, `2 weeks`)
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
   todo
   ```

## Usage

### Basic Navigation

Navigate through the application using these keybindings:

- `Tab`/`1`/`2`/`3`: Switch between Workspace, Archived Workspace, and Todo List panels
- `Arrow Keys`/`j`/`k`: Navigate up and down in lists
- `Enter`/`l`: Enter a workspace to view its tasks
- `h`: Go back from Todo List to Workspace
- `a`: Add new item (workspace or task depending on focus)
- `i`: Add child item (sub-workspace or sub-task)
- `x`: Delete selected item
- `r`: Rename selected item
- `f`/`/`: Filter/search tasks
- `?`: Show help screen
- `Ctrl+s`: Save data manually
- `q`: Quit application
- `Esc`: Exit help screen/search mode

### Task Management

Manage your tasks efficiently with these commands:

- `t`: Mark task as `Todo`
- `p`: Mark task as `In Progress`
- `c`: Mark task as `Completed`
- `d`: Mark task as `Deprecated`
- `D`: Set due date for task

> **Tip**
> There are three ways to set a due date:
>
> - Use `Ctrl+o` to open the calendar and press `Enter` to select
> - Enter a date directly like `2025-08-19`
> - Enter remaining time like `1 day` `2 days` `3 weeks` `4 months`

### Workspace Management

Organize your work with workspaces:

- `A`: Archive current workspace
- `R`: Recover archived workspace

### Data Storage

The application automatically saves data to `~/.todo/data.json`. This file contains all your workspaces, tasks, and their statuses.

> **Note**
> To keep the program small and ensure convenient and manageable data storage, a `.json` file is used to store data, which allows direct modification and management of data (though not strictly necessary).

## Keybindings Reference

### Navigation Keys

| Key | Action |
|-----|--------|
| `Tab`/`1`/`2`/`3` | Switch focus between panels |
| `j`/`k` or `Arrow Keys` | Navigate up/down |
| `Enter`/`l` | Enter workspace |
| `h` | Return to workspace from todo list |
| `a` | Add item |
| `i` | Add child item |
| `x` | Delete item |
| `r` | Rename item |
| `f` or `/` | Filter/search |
| `?` | Show help |
| `Ctrl+s` | Save data |
| `q` or `Esc` | Quit |

### Task Status Keys

| Key | Status |
|-----|--------|
| `t` | Todo |
| `p` | In Progress |
| `c` | Completed |
| `d` | Deprecated |
| `D` | Set Due Date |

### Workspace Keys

| Key | Action |
|-----|--------|
| `A` | Archive workspace |
| `R` | Recover workspace |

## To-Do

- [ ] Implement **urgency level** setting
- [ ] Implement **sorting methods** to change task sorting, or sort by **urgency level**, **remaining time**
- [ ] Consider using a **database** for data storage

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request or open an Issue.

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.
