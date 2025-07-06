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
}

#[derive(Debug, Clone)]
pub enum CurrentFocus {
    Workspace,
    TodoList,
}

#[derive(Debug)]
pub enum CurrentMode {
    Normal,
    Insert,
}
