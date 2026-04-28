# API Reference

## 1. Type Definitions

### 1.1 Core Types

```rust
// app.rs
pub struct App {
    pub appstate: Arc<Mutex<AppState>>,
}

// appstate.rs
pub struct AppState {
    pub current_focus: CurrentFocus,
    pub current_mode: CurrentMode,
    pub exit: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum CurrentFocus {
    Workspace,
    TodoList,
    ArchivedWorkspace,
}

#[derive(Debug, Clone, Copy)]
pub enum CurrentMode {
    Normal,
    Insert,
    Search,
    Help,
    Sort,
}
```

### 1.2 Message Types

```rust
// appstate.rs
#[derive(Debug)]
pub enum Message {
    Update,
    ChangeMode(CurrentMode),
    ChangeFocus(CurrentFocus),
    SelectWorkspace,
    AddItem,
    AddChild,
    DeleteItem,
    MoveUp,
    MoveDown,
    Exit,
    Archive,
    Recovery,
    Complete,
    InProcess,
    Todo,
    Deprecated,
    Rename,
    Filter,
    SearchMsg(Search_event::SearchEvent),
    Help,
    ExitHelp,
    Due,
    SaveData,
    IncreseUrgency,
    DecreseUrgency,
    Sort,
}

// ui.rs
#[derive(Debug)]
pub enum UiMessage {
    Update,
    UpdateUi,
    SaveData,
    WAction(WidgetAction),
}

#[derive(Debug)]
pub enum WidgetAction {
    AddWorkspace,
    AddWorkspaceChild,
    AddTask,
    AddTaskChild,
    SelectUp,
    SelectDown,
    FocusWorkspace,
    FocusTodolist,
    FocusArchivedWorkspace,
    EnterWorkspace,
    EnterArchivedWorkspace,
    DeleteWorkspace,
    DeleteArchivedWorkspace,
    DeleteTask,
    MarkTaskStatus(TaskStatus),
    ArchiveWS,
    RecoveryWS,
    Rename(CurrentFocus),
    Filter,
    ExitFilter,
    Help,
    ExitHelp,
    Due,
    IncreseUrgency,
    DecreseUrgency,
    Sort,
}

#[derive(Debug)]
pub enum SearchEvent {
    Next,
    Previous,
    Exit,
}

#[derive(Debug)]
pub enum SelectBF {
    Back,
    Forward,
}
```

## 2. Data Models

### 2.1 Workspace

```rust
// ui/workspacewidget.rs
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub children: Vec<Rc<RefCell<Workspace>>>,
}

pub enum WorkspaceType {
    Normal,
    Archived,
}

pub struct WorkspaceWidget {
    pub workspaces: Vec<Rc<RefCell<Workspace>>>,
    pub current_workspace: Option<Rc<RefCell<Workspace>>>,
    pub current_workspace_id: Option<String>,
    pub focused: bool,
    pub ws_state: ListState,
    pub ws_type: WorkspaceType,
}
```

### 2.2 Task

```rust
// ui/todolistwidget.rs
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub due_date: Option<String>,
    pub urgency: i32,
    pub children: Vec<Rc<RefCell<Task>>>,
    pub tags: Vec<String>,  // FUTURE
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Finished,
    Deprecated,
}

pub struct TodoList {
    pub id: String,
    pub tasks: Vec<Rc<RefCell<Task>>>,
    pub current_task: Option<Rc<RefCell<Task>>>,
}

pub struct TodoWidget {
    pub todolists: Vec<Rc<RefCell<TodoList>>>,
    pub current_todolist: Option<Rc<RefCell<TodoList>>>,
}
```

### 2.3 Data Persistence

```rust
// data.rs
pub struct Datas {
    pub workspace: WorkspaceWidget,
    pub todolist: TodoWidget,
    pub archived_ws: WorkspaceWidget,
}
```

## 3. Error Types

```rust
// errors.rs
#[derive(Debug)]
pub enum Errors {
    AppError,
    UiError,
    WriteError,
    LoadError,
}
```

## 4. Function Signatures

### 4.1 App

```rust
// main.rs
pub fn main()

// app.rs
impl App {
    pub fn new() -> Self
    pub fn run(&self) -> Result<(), Errors>
}

async fn handle_keyevt(
    tx: mpsc::Sender<Message>,
    input_tx: mpsc::Sender<KeyEvent>,
    appstate: Arc<Mutex<AppState>>,
)

async fn handle_msg(
    rx: mpsc::Receiver<Message>,
    ui_tx: mpsc::Sender<UiMessage>,
    appstate: Arc<Mutex<AppState>>,
)
```

### 4.2 Data

```rust
// data.rs
pub fn save_data(path: &Path, datas: &Datas) -> Result<(), Errors>
pub fn load_data(path: &Path) -> Result<Datas, Errors>
```

### 4.3 UI

```rust
// ui.rs
impl Ui {
    pub fn new(
        ui_rx: mpsc::Receiver<UiMessage>,
        input_rx: mpsc::Receiver<KeyEvent>,
    ) -> Self
    
    pub fn update(&mut self, f: &mut Frame)
    pub fn refresh_current(&mut self)
    
    pub async fn get_input(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<KeyEvent>>>,
        terminal: &mut DefaultTerminal,
        title: String,
    ) -> String
    
    pub async fn input_due_date(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<KeyEvent>>>,
        terminal: &mut DefaultTerminal,
        title: String,
        origin_due: Option<NaiveDate>,
    ) -> String
    
    pub async fn delete_item(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<KeyEvent>>>,
        terminal: &mut DefaultTerminal,
    ) -> bool
    
    pub async fn confirm_delete(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<KeyEvent>>>,
        terminal: &mut DefaultTerminal,
        target: CurrentFocus,
    ) -> bool
    
    pub async fn filter_find(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<KeyEvent>>>,
        terminal: &mut DefaultTerminal,
    ) -> String
    
    pub async fn handle_uimsg(
        &mut self,
        terminal: &mut DefaultTerminal,
        appstate: Arc<Mutex<AppState>>,
    )
}
```

### 4.4 Workspace Widget

```rust
// ui/workspacewidget.rs
impl Workspace {
    pub fn new(name: String) -> Self
    pub fn has_todolist(&self, todolist: &TodoWidget) -> bool
}

impl WorkspaceWidget {
    pub fn new(ws_type: WorkspaceType) -> Self
    pub fn add_workspace(&mut self, workspace: Rc<RefCell<Workspace>>)
    pub fn add_child_workspace(&mut self, workspace: Rc<RefCell<Workspace>>)
    pub fn refresh_current(&mut self)
    pub fn get_selected_bf(&mut self, bf: SelectBF) -> Option<Rc<RefCell<Workspace>>>
    pub fn delete_item(
        workspaces: &mut Vec<Rc<RefCell<Workspace>>>,
        target: &Rc<RefCell<Workspace>>,
    )
}
```

### 4.5 Todo Widget

```rust
// ui/todolistwidget.rs
impl Task {
    pub fn new(title: String, due_date: Option<String>) -> Self
    pub fn is_target(&self, pattern: String) -> bool
}

impl TodoList {
    pub fn new(id: String) -> Self
    pub fn add_task(&mut self, task: Rc<RefCell<Task>>)
    pub fn add_child_task(&mut self, task: Rc<RefCell<Task>>)
    pub fn get_selected_bf(&mut self, bf: SelectBF) -> Option<Rc<RefCell<Task>>>
}

impl TodoWidget {
    pub fn new() -> Self
    pub fn add_list(&mut self, list: Rc<RefCell<TodoList>>)
    pub fn delete_list(&mut self, target_id: String)
    pub fn change_current_list(&mut self, workspace: &Option<Rc<RefCell<Workspace>>>)
    pub fn get_selected_bf(&mut self, bf: SelectBF) -> Option<Rc<RefCell<Task>>>
}
```

## 5. Trait Definitions

```rust
// ui.rs
pub trait SelectAction<T> {
    fn get_selected_bf(&mut self, bf: SelectBF) -> Option<Rc<RefCell<T>>>
    fn get_flattened(target: &Vec<Rc<RefCell<T>>>) -> Vec<Rc<RefCell<T>>>;
}
```

## 6. Keymap Definitions

```rust
// ui/keymap.rs
pub struct Keymap {
    pub key: String,
    pub desc: String,
    pub detailed: String,
}

pub struct KeymapWidget {
    pub focus: CurrentFocus,
    pub mode: CurrentMode,
    pub general_hint: Vec<Keymap>,
    pub workspace_hint: Vec<Keymap>,
    pub tasklist_hint: Vec<Keymap>,
    pub archived_ws_hint: Vec<Keymap>,
    pub sort_hint: Vec<Keymap>,
}
```

## 7. Widget Definitions

```rust
// ui/calendarwidget.rs
pub struct CalendarWidget {
    pub cursor: NaiveDate,
    pub selected: NaiveDate,
}

impl CalendarWidget {
    pub fn new() -> Self
    pub fn move_left(&mut self)
    pub fn move_right(&mut self)
    pub fn move_up(&mut self)
    pub fn move_down(&mut self)
}

// ui/helpwidget.rs
pub struct HelpWidget {
    pub keymap: KeymapWidget,
    pub scroll: usize,
    pub scroll_max: usize,
    pub state: ListState,
}

// ui/prompt.rs
pub struct PromptWidget {
    pub desc: String,
}
```

## 8. Constants

```rust
// Data file location
const DATA_PATH: &str = ".todo/data.json";

// Default urgency
const DEFAULT_URGENCY: i32 = 0;
const MIN_URGENCY: i32 = -10;
const MAX_URGENCY: i32 = 10;
```

## 9. Module Public API

```rust
// src/main.rs
pub mod app;

// src/app.rs
pub mod appstate;
pub mod data;
pub mod errors;
pub mod ui;

pub struct App { ... }

// src/app/appstate.rs
pub struct AppState { ... }
pub enum Message { ... }
pub enum CurrentFocus { ... }
pub enum CurrentMode { ... }

// src/app/data.rs
pub struct Datas { ... }
pub fn save_data(...) -> Result<...>
pub fn load_data(...) -> Result<...>

// src/app/errors.rs
pub enum Errors { ... }

// src/app/ui.rs
pub struct Ui { ... }
pub enum UiMessage { ... }
pub enum WidgetAction { ... }
pub enum SearchEvent { ... }
pub enum SelectBF { ... }
pub trait SelectAction<T> { ... }
```