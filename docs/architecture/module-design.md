# Module Design

## 1. Module Structure

```
src/
├── main.rs              # Entry point (58 lines)
├── app.rs              # Main coordinator (619 lines)
│   ├── handle_keyevt()  # Key → Message
│   ├── handle_msg()    # Message → UiMessage
│   └── handle_uimsg()  # UiMessage → UI update
│
├── app/
│   ├── appstate.rs     # State management
│   ├── data.rs        # Persistence
│   ├── errors.rs      # Error types
│   └── ui/
│       ├── ui.rs            # UI orchestration
│       ├── workspacewidget.rs  # Workspace list
│       ├── todolistwidget.rs  # Task list
│       ├── calendarwidget.rs   # Date picker
│       ├── helpwidget.rs     # Help overlay
│       ├── keymap.rs        # Key hints
│       └── prompt.rs       # Status bar
```

## 2. Module Responsibilities

### 2.1 main.rs

| Responsibility | API |
|---------------|-----|
| Entry point | `main()` |
| Create App | `App::new()` |
| Run App | `App::run()` |
| Error handling | `Result<(), Errors>` |

```mermaid
graph LR
    main["main.rs"] --> app["app.rs"]
    app -->|new| App["App instance"]
    app -->|run| Terminal["ratatui"]
    Terminal -->|success| End["println Exit"]
    Terminal -->|error| Err["println error"]
```

### 2.2 app.rs

| Responsibility | Functions |
|---------------|----------|
| Thread creation | `run()` |
| Key handling | `handle_keyevt()` |
| Message handling | `handle_msg()` |
| UI coordination | `handle_uimsg()` |

```mermaid
graph TD
    subgraph App["app.rs"]
        run["pub fn run()"]
        handle_key["handle_keyevt()"]
        handle_msg["handle_msg()"]
        handle_ui["handle_uimsg()"]
    end
    
    run --> T1["spawn KeyEvent thread"]
    run --> T2["spawn Message thread"]
    run --> T3["spawn UI thread"]
    
    T1 --> handle_key
    handle_key -->|Message| handle_msg
    handle_msg -->|UiMessage| handle_ui
```

#### handle_keyevt()

| Aspect | Detail |
|-------|-------|
| Input | `KeyEvent` from `crossterm::event::read()` |
| Output | `Message` to message handler, `KeyEvent` to input handler |
| State | Read `current_focus`, `current_mode` from `AppState` |
| Modes | Normal, Insert, Search, Help, Sort |

```rust
async fn handle_keyevt(
    tx: mpsc::Sender<Message>,
    input_tx: mpsc::Sender<KeyEvent>,
    appstate: Arc<Mutex<AppState>>,
) {
    // Loop: read key → match mode → send message
}
```

#### handle_msg()

| Aspect | Detail |
|-------|-------|
| Input | `Message` from channel |
| Output | `UiMessage` to UI thread |
| State | Write `current_mode`, `current_focus` to `AppState` |
| Actions | Map Message to UiMessage |

```rust
async fn handle_msg(
    rx: mpsc::Receiver<Message>,
    ui_tx: mpsc::Sender<UiMessage>,
    appstate: Arc<Mutex<AppState>>,
) {
    // Loop: recv message → match → send ui message
}
```

#### handle_uimsg()

| Aspect | Detail |
|-------|-------|
| Input | `UiMessage` from channel |
| Output | Terminal render |
| State | Read/Write widget state |
| Actions | Execute widget actions, render |

```rust
async fn handle_uimsg(
    terminal: &mut DefaultTerminal,
    appstate: Arc<Mutex<AppState>>,
) {
    // Loop: recv ui message → execute → render
}
```

### 2.3 appstate.rs

| Responsibility | Types |
|---------------|-------|
| Global state | `AppState` |
| Focus management | `CurrentFocus` |
| Mode management | `CurrentMode` |
| Message types | `Message` |

```mermaid
graph BT
    AppState["struct AppState"]
    CurrentFocus["enum CurrentFocus"]
    CurrentMode["enum CurrentMode"]
    Message["enum Message"]
    
    AppState --> current_focus["current_focus: CurrentFocus"]
    AppState --> current_mode["current_mode: CurrentMode"]
    AppState --> exit["exit: bool"]
    
    CurrentFocus --> Workspace["Workspace"]
    CurrentFocus --> TodoList["TodoList"]
    CurrentFocus --> Archived["ArchivedWorkspace"]
    
    CurrentMode --> Normal["Normal"]
    CurrentMode --> Insert["Insert"]
    CurrentMode --> Search["Search"]
    CurrentMode --> Help["Help"]
    CurrentMode --> Sort["Sort"]
```

### 2.4 data.rs

| Responsibility | Functions |
|---------------|----------|
| Data container | `Datas` struct |
| Load data | `load_data(path)` |
| Save data | `save_data(path, datas)` |

```mermaid
graph LR
    subgraph Load["Load Flow"]
        L1["path exists?"]
        L2["read file"]
        L3["serde_json::from_str"]
        L4["Datas"]
    end
    
    subgraph Save["Save Flow"]
        S1["Datas"]
        S2["serde_json::to_string_pretty"]
        S3["fs::write"]
    end
    
    L1 -->|yes| L2 --> L3 --> L4
    L1 -->|no| Create["create dir<br/>return Datas::default"]
    Create --> L4
```

### 2.5 ui.rs

| Responsibility | Methods |
|---------------|--------|
| Widget management | `Ui` struct |
| Rendering | `update(f)` |
| Input dialogs | `get_input()`, `input_due_date()` |
| Confirmation | `delete_item()`, `confirm_delete()` |
| Search | `filter_find()` |

```mermaid
graph TD
    subgraph Ui["struct Ui"]
        widgets["workspace: WorkspaceWidget<br/>todolist: TodoWidget<br/>archived_ws: WorkspaceWidget<br/>helpwidget: HelpWidget<br/>prompt: PromptWidget"]
        
        methods["get_input()<br/>input_due_date()<br/>delete_item()<br/>confirm_delete()<br/>filter_find()<br/>refresh_current()<br/>handle_uimsg()"]
    end
    
    methods -->|read/write| widgets
```

### 2.6 workspacewidget.rs

| Responsibility | Types |
|---------------|-------|
| Workspace data | `Workspace` struct |
| Widget rendering | `WorkspaceWidget` struct |
| Workspace type | `WorkspaceType` enum |

```mermaid
graph BT
    Workspace["struct Workspace"]
    WorkspaceWidget["struct WorkspaceWidget"]
    WorkspaceType["enum WorkspaceType"]
    
    WorkspaceWidget --> workspaces["Vec&ltRc&ltRefCell&ltWorkspace&gt&gt;&gt;"]
    WorkspaceWidget --> current_workspace["Option&ltRc&ltRefCell&ltWorkspace&gt&gt;&gt;"]
    WorkspaceWidget --> focused["bool"]
    
    Workspace --> id["id: String"]
    Workspace --> name["name: String"]
    Workspace --> children["children: Vec"]
    WorkspaceWidget --> ws_state["ListState"]
```

### 2.7 todolistwidget.rs

| Responsibility | Types |
|---------------|-------|
| Task data | `Task` struct |
| Task status | `TaskStatus` enum |
| Task list | `TodoList` struct |
| Widget | `TodoWidget` struct |

```mermaid
graph BT
    Task["struct Task"]
    TaskStatus["enum TaskStatus"]
    TodoList["struct TodoList"]
    TodoWidget["struct TodoWidget"]
    
    Task --> title["title: String"]
    Task --> status["status: TaskStatus"]
    Task --> due_date["due_date: Option&ltString&gt"]
    Task --> urgency["urgency: i32"]
    Task --> children["children: Vec&ltRc&ltRefCell&ltTask&gt&gt;&gt;"]
    Task -->|NEW| tags["tags: Vec&ltString&gt"]
    
    TaskStatus --> Todo["Todo"]
    TaskStatus --> InProgress["InProgress"]
    TaskStatus --> Completed["Finished"]
    TaskStatus --> Deprecated["Deprecated"]
    
    TodoList --> tasks["Vec&ltRc&ltRefCell&ltTask&gt&gt;&gt;"]
    TodoList --> current_task["Option&ltRc&ltRefCell&ltTask&gt&gt;&gt;"]
    TodoWidget --> todolists["Vec&ltRc&ltRefCell&ltTodoList&gt&gt;&gt;"]
    TodoWidget --> current_todolist["Option&ltRc&ltRefCell&ltTodoList&gt&gt;&gt;"]
```

### 2.8 calendarwidget.rs

| Responsibility | Features |
|---------------|---------|
| Calendar display | `CalendarWidget` struct |
| Navigation | `move_left()`, `move_right()`, `move_up()`, `move_down()` |
| Selection | `cursor: NaiveDate` |

```mermaid
graph LR
    CalendarWidget["struct CalendarWidget"]
    cursor["cursor: NaiveDate"]
    selected["selected: NaiveDate"]
    
    CalendarWidget -->|move_left| left["-1 day"]
    CalendarWidget -->|move_right| right["+1 day"]
    CalendarWidget -->|move_up| up["-7 days"]
    CalendarWidget -->|move_down| down["+7 days"]
```

### 2.9 helpwidget.rs

| Responsibility | Features |
|---------------|---------|
| Help display | `HelpWidget` struct |
| Key hints | `KeymapWidget` struct |
| Scroll | `scroll: usize`, `state: ListState` |

```mermaid
graph TD
    HelpWidget["struct HelpWidget"]
    KeymapWidget["struct KeymapWidget"]
    
    HelpWidget --> keymap["keymap: KeymapWidget"]
    HelpWidget --> scroll["scroll: usize"]
    HelpWidget --> state["ListState"]
    
    KeymapWidget --> focus["CurrentFocus"]
    KeymapWidget --> mode["CurrentMode"]
    KeymapWidget --> general_hint["Vec&ltKeymap&gt;"]
    KeymapWidget --> sort_hint["Vec&ltKeymap&gt;"]
```

## 3. Public API Summary

### 3.1 Exported Types

| Module | Type | Visibility |
|--------|------|----------|
| `app::` | `App` | `pub` |
| `appstate::` | `AppState`, `Message`, `CurrentFocus`, `CurrentMode` | `pub` |
| `data::` | `Datas` | `pub` |
| `errors::` | `Errors` | `pub` |
| `ui::` | `Ui`, `UiMessage`, `WidgetAction` | `pub` |
| `ui::` | `SearchEvent`, `SelectBF` | `pub` |
| `ui::workspacewidget::` | `Workspace`, `WorkspaceWidget`, `WorkspaceType` | `pub` |
| `ui::todolistwidget::` | `Task`, `TaskStatus`, `TodoList`, `TodoWidget` | `pub` |

### 3.2 Function Signatures

```rust
// main.rs
pub fn main() { ... }

// app.rs
impl App {
    pub fn new() -> Self { ... }
    pub fn run(&self) -> Result<(), Errors> { ... }
}

// data.rs
pub fn save_data(path: &Path, datas: &Datas) -> Result<(), Errors> { ... }
pub fn load_data(path: &Path) -> Result<Datas, Errors> { ... }

// ui.rs
impl Ui {
    pub async fn get_input(...) -> String { ... }
    pub async fn input_due_date(...) -> String { ... }
    pub async fn delete_item(...) -> bool { ... }
    pub async fn confirm_delete(...) -> bool { ... }
    pub async fn filter_find(...) -> String { ... }
    pub async fn handle_uimsg(...) { ... }
    pub fn update(&mut self, f: &mut Frame) { ... }
}
```

## 4. Code Statistics

| File | Lines | Responsibility |
|------|-------|-------------|
| `main.rs` | 58 | Entry point |
| `app.rs` | 619 | Coordination (TO DO: shrink) |
| `appstate.rs` | 221 | State management |
| `data.rs` | 148 | Persistence |
| `errors.rs` | 46 | Error types |
| `ui.rs` | 1500+ | UI orchestration (TO DO: split) |
| `workspacewidget.rs` | ~300 | Workspace widget |
| `todolistwidget.rs` | ~400 | Task widget |
| `calendarwidget.rs` | ~150 | Calendar widget |
| `helpwidget.rs` | ~100 | Help widget |
| `keymap.rs` | 173 | Key hints |
| `prompt.rs` | ~50 | Status bar |

**Total**: ~3700 lines

## 5. Refactoring Targets

| Priority | File | Target | Action |
|---------|------|--------|--------|
| HIGH | `app.rs` | <200 lines | Extract message handling to services |
| HIGH | `ui.rs` | <500 lines | Split into `ui/` module |
| MEDIUM | `workspacewidget.rs` | Extract sort logic | New `services/sort.rs` |
| MEDIUM | `todolistwidget.rs` | Add tags support | Add `tags` field |
| LOW | `keymap.rs` | Simplify | Move to `ui/` |