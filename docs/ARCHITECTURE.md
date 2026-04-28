# Architecture Design

## 1. Current Architecture Analysis

### Strengths
- Message-passing between threads (keyevt → msg → UI)
- Clean separation of concerns
- Stateful widgets with ListState

### Issues
- `app.rs` too large (619 lines)
- UI logic mixed with business logic
- Action/WidgetAction duplication
- No clear model layer

---

## 2. Recommended Pattern

**Component-Based Architecture with Action Dispatch**

| Layer | Responsibility | Current Location |
|-------|---------------|---------------|
| **UI Layer** | Rendering, widget logic | `ui/*.rs` |
| **Business Logic** | State mutation, actions | `app.rs` |
| **Model Layer** | Data structures | `models/` |
| **Service Layer** | Persistence | `services/` |

---

## 3. Target Module Structure

```
src/
├── main.rs                    # Entry point
├── lib.rs                    # Module exports
│
├── app.rs                    # App + event loop (瘦身后)
│   ├── handle_key_event()     # Key → Action
│   ├── handle_action()       # Action → State mutation
│   └── handle_uimsg()       # Action → UI update
│
├── models/                   # Data structures
│   ├── mod.rs
│   ├── workspace.rs         # Workspace { id, name, children, parent_id }
│   ├── task.rs            # Task { id, title, status, due_date, urgency, subtasks, tags }
│   ├── input_mode.rs      # InputMode (Normal, Insert, Search, Sort, Help)
│   ├── focus_panel.rs     # FocusPanel (Workspace, TodoList, Archived)
│   └── task_status.rs     # TaskStatus (Todo, InProgress, Completed, Deprecated)
│
├── actions.rs              # All user intents
│
├── services/              # Business logic
│   ├── mod.rs
│   └── persistence.rs    # load(), save()
│
└── ui/                   # UI components
    ├── mod.rs
    ├── layout.rs        # Layout constraints
    ├── style.rs        # Colors, themes
    ├── keymap.rs       # Keybinding hints
    └── widgets/
        ├── mod.rs
        ├── workspace.rs   # Workspace list widget
        ├── todolist.rs    # Task list widget
        ├── help.rs        # Help overlay
        ├── calendar.rs   # Date picker
        └── status_bar.rs # Bottom line
```

---

## 4. State Management

```rust
// app.rs
pub struct App {
    // Models
    pub workspaces: WorkspaceWidget,      // From ui/
    pub todolist: TodoWidget,           // From ui/
    pub archived_ws: WorkspaceWidget,    // From ui/
    
    // State
    pub mode: InputMode,
    pub focus: FocusPanel,
    pub exit: bool,
}

impl App {
    // Key → Action
    fn handle_key_event(&mut self, key: KeyEvent) -> Action { ... }
    
    // Action → State mutation + UI command
    fn handle_action(&mut self, action: Action) -> Option<UiCommand> { ... }
}
```

---

## 5. Action Dispatch Pattern

```rust
// KeyEvent → Action (in handle_key_event)
match (self.mode, key.code) {
    (Normal, KeyCode::Char('j')) => Action::MoveDown,
    (Normal, KeyCode::Char('a')) => Action::AddItem,
    // ...
    (Insert, _) => Action::PassToInput,
}

// Action → UI Command
fn handle_action(&mut self, action: Action) -> Option<UiCommand> {
    match action {
        Action::AddItem => {
            self.mode = InputMode::Insert;
            Some(UiCommand::ShowAddDialog)
        }
        Action::MoveDown => {
            self.move_selection(Direction::Down);
            None
        }
        // ...
    }
}
```

---

## 6. Migration Plan

### Phase 1: Extract Models
- [ ] Create `models/` directory
- [ ] Move `Task`, `Workspace` from ui modules
- [ ] Add `tags: Vec<String>` to Task

### Phase 2: Extract Actions
- [ ] Create `actions.rs` with all Action variants
- [ ] Remove `Message`/`UiMessage` duplication
- [ ] Simplify to single Action enum

### Phase 3: Refactor App
- [ ] Shrink `app.rs` to event loop
- [ ] Extract business logic to service functions
- [ ] Clear input/output for each function

### Phase 4: UI Refinement
- [ ] Organize widgets into `ui/widgets/`
- [ ] Add `layout.rs` and `style.rs`
- [ ] Separate rendering from state

---

## 7. Key Design Decisions

| Decision | Choice | Reason |
|----------|--------|--------|
| Data storage | JSON (keep) | Simplicity, user requirement |
| State location | App struct | Single source of truth |
| Thread model | Keep current | Works well |
| Error handling | Keep enum | Consistent |

---

## 8. Reference Projects

| Project | Pattern Used |
|---------|-------------|
| sebyx07/rust-ratatui-todos | Component-based |
| sheepla/ratatodo | Flux-inspired action dispatch |
| ratatui official examples | StatefulWidget pattern |

---

## 9. Next Steps

1. **Current priority**: Implement Sort Mode with current architecture
2. **After Sort Mode**: Refactor based on this design
3. **Future**: Add Tag System following this structure