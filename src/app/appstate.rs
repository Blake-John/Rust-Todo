/// Structure for app state
///
/// # Fields
///
/// - `current_focus` ([`CurrentFocus`]) - state which is focused
/// - `current_mode` ([`CurrentMode`]) - state which mode is active
/// - `exit` (`bool`) - whether the app should exit
///
/// # Examples
///
/// ```
/// use crate::app::appstate::AppState;
///
/// let appstate = AppState::new();
/// ```
#[derive(Debug)]
pub struct AppState {
    pub current_focus: CurrentFocus,
    pub current_mode: CurrentMode,
    pub exit: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_focus: CurrentFocus::Workspace,
            current_mode: CurrentMode::Normal,
            exit: false,
        }
    }
}

/// Message of the application
///
/// # Variants
///
/// - `Update` - Update the application ui
/// - `ChangeMode(CurrentMode)` - change the mode of the application
/// - `ChangeFocus(CurrentFocus)` - change the focus of the application
/// - `SelectWorkspace` - enter a workspace to create task
/// - `AddItem` - add a workspace or add a task
/// - `AddChild` - add a sub ws or sub task
/// - `DeleteItem` - delete a workspace or a task
/// - `MoveUp` - move the cursor up / select the item
/// - `MoveDown` - move the cursor down / select the item
/// - `Exit` - exit the application
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
    Complete,
    InProcess,
    Todo,
    Deprecated,
    Rename,
}

/// State of which is focused
#[derive(Debug, Clone)]
pub enum CurrentFocus {
    Workspace,
    TodoList,
}

/// State of which mode is active
#[derive(Debug)]
pub enum CurrentMode {
    Normal,
    Insert,
}
