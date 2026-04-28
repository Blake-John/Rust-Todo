# Rust Todo Application - AGENTS.md

## Project Overview

A terminal-based todo application built with Rust and Ratatui, featuring workspace management, task tracking, nested subtasks, due dates, urgency levels, and TUI interface.

## Architecture

### Source Structure

```
src/
├── main.rs           # Entry point
├── app.rs           # Main app logic, message handling
├── app/
│   ├── appstate.rs  # AppState, CurrentFocus, CurrentMode, Message
│   ├── data.rs      # Data persistence (JSON load/save)
│   ├── errors.rs    # Error types
│   └── ui/
│       ├── ui.rs           # Main UI orchestration
│       ├── workspacewidget.rs  # Workspace list rendering
│       ├── todolistwidget.rs    # Task list rendering
│       ├── calendarwidget.rs   # Date picker
│       ├── helpwidget.rs       # Help screen
│       ├── keymap.rs          # Keyboard hints widget
│       └── prompt.rs         # Status bar
```

### Key Concepts

- **Workspace**: Container for tasks, supports nesting (parent/child)
- **Task**: Has status (todo/in-progress/completed/deprecated), due date, urgency, subtasks
- **Focus Panels**: Workspace | TodoList | ArchivedWorkspace
- **Modes**: Normal | Insert | Search | Help | Sort

### Documentation

| File | Description |
|------|-------------|
| `docs/architecture/` | Full architecture documentation |
| `docs/TASKS.md` | **Current development tasks** |
| `docs/PROCESS.md` | Development progress log |

## Development Workflow

### ⚠️ Before Every Development Task

1. **Read TASKS.md** - Get current task ID and details
2. **Read WORKFLOW.md** - Follow the development process
3. **Update PROCESS.md** - Record your progress

### Development Process

```
Research → Design → Implement → Test → Review → Commit
```

| Phase | Actions |
|-------|---------|
| **Research** | Read code, external research, clarify requirements |
| **Design** | Design implementation, verify with user |
| **Implement** | Write code, surgical changes, match style |
| **Test** | Build + manual test |
| **Review** | Code review, address issues |
| **Commit** | Write commit message |

### Task Workflow

```
1. Check docs/TASKS.md for current task (look for PENDING with HIGH priority)
2. Read task details (checklist, exit criteria)
3. Follow WORKFLOW.md phases for this task
4. Update docs/PROCESS.md as you progress
5. Mark task complete in TASKS.md
6. Move to next task
```

## Current Development Priority

### Phase 1: Sort Mode (HIGH) - ✅ COMPLETED

| ID | Task | Status |
|----|------|--------|
| T001 | Research - Read existing code | ✅ |
| T002 | Implement sort functions | ✅ |
| T003 | Test (build + manual) | ✅ |
| T004 | Code Review | ✅ |
| T005 | Cleanup | ✅ |

### Bug Fixes Session

| ID | Issue | Status |
|----|------|--------|
| B001 | JSON ws_type missing | ✅ FIXED |
| B002 | archived_ws missing | ✅ FIXED |
| B003 | Sort mode deadlock | ✅ FIXED |
| B004 | Archived display wrong | ✅ FIXED |

### Phase 2: Tag System (MEDIUM)

| ID | Task | Status |
|----|------|--------|
| T006-T010 | Research, Design, Implement, Test, Review | PENDING |

### Phase 3: Cleanup (LOW)

| ID | Task | Status |
|----|------|--------|
| T011-T012 | Remove TODOs, Update README | PENDING |

## Keybindings Reference

| Key | Action |
|-----|--------|
| Tab/1/2/3 | Switch focus panels |
| j/k, ↑/↓ | Navigate |
| Enter/l | Enter workspace |
| h | Return to workspace |
| a | Add item |
| i | Add child item |
| x | Delete |
| r | Rename |
| t/p/c/d | Task status (todo/in-progress/completed/deprecated) |
| D | Set due date |
| +/= , -/_ | Urgency +/- |
| f/ / | Filter |
| ? | Help |
| s | Sort |
| q | Quit |

## Dependencies

See `Cargo.toml`:
- ratatui 0.29.0 (TUI framework)
- tokio 1.47.1 (async runtime)
- serde/serde_json (serialization)
- chrono 0.4.41 (dates)
- uuid 1.18.0 (unique IDs)
- crossterm 0.29.0 (terminal input)
- regex 1.11.1 (text matching)
- tui-textarea 0.7.0 (text input)

## Notes for AI Agent

1. **DO NOT** change data storage format (keep JSON)
2. **ALWAYS** use `.unwrap()` after `serde_json::to_string_pretty()` as currently done (risk: panic)
3. **KEEP** all existing keyboard shortcuts working
4. **MATCH** existing code style (camelCase, detailed comments)
5. **TEST** after changes: `cargo build --release`
6. **ALWAYS** check TASKS.md before starting work
7. **ALWAYS** follow WORKFLOW.md for development process
8. **ALWAYS** update PROCESS.md to record progress