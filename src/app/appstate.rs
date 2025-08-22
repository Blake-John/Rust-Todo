//! Application state management module
//!
//! This module defines the core state structures that control the application's behavior,
//! including focus management, mode states, and message passing between components.

use crate::app::ui::SearchEvent;

/// Structure for app state
///
/// This struct holds the global state of the application, tracking which component
/// currently has focus, what mode the application is in, and whether the application
/// should exit.
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
    /// The currently focused component of the UI
    pub current_focus: CurrentFocus,
    /// The current mode of the application
    pub current_mode: CurrentMode,
    /// Flag indicating whether the application should exit
    pub exit: bool,
}

impl AppState {
    /// Creates a new AppState with default values
    ///
    /// Initializes the application state with:
    /// - Focus on the Workspace component
    /// - Normal mode
    /// - Exit flag set to false
    ///
    /// # Returns
    ///
    /// A new instance of [`AppState`]
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::app::appstate::AppState;
    ///
    /// let appstate = AppState::new();
    /// assert_eq!(appstate.current_focus, crate::app::appstate::CurrentFocus::Workspace);
    /// assert_eq!(appstate.current_mode, crate::app::appstate::CurrentMode::Normal);
    /// assert_eq!(appstate.exit, false);
    /// ```
    pub fn new() -> Self {
        Self {
            current_focus: CurrentFocus::Workspace,
            current_mode: CurrentMode::Normal,
            exit: false,
        }
    }
}

impl Default for AppState {
    /// Creates a default AppState instance
    ///
    /// This is equivalent to calling [`AppState::new()`]
    ///
    /// # Returns
    ///
    /// A default instance of [`AppState`]
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::app::appstate::AppState;
    ///
    /// let appstate = AppState::default();
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

/// Message of the application
///
/// Enum representing all possible messages that can be sent between components
/// in the application. These messages are used to communicate actions and state
/// changes between the UI, event handlers, and business logic.
///
/// Messages are processed by the main application loop and dispatched to appropriate
/// handlers. They represent user interactions, system events, and state changes
/// that drive the application's behavior.
///
/// # Message Flow
///
/// 1. User input (keypresses) are converted to Messages by the event handler
/// 2. Messages are sent through a channel to the main application loop
/// 3. The main loop processes each message and updates application state
/// 4. UI updates are triggered based on the processed messages
///
/// # Examples
///
/// ```
/// use crate::app::appstate::{Message, CurrentMode, CurrentFocus};
/// use crate::app::ui::SearchEvent;
///
/// // Example messages that might be sent in the application
/// let update_msg = Message::Update;
/// let mode_msg = Message::ChangeMode(CurrentMode::Insert);
/// let focus_msg = Message::ChangeFocus(CurrentFocus::TodoList);
/// let search_msg = Message::SearchMsg(SearchEvent::Next);
/// ```
#[derive(Debug)]
pub enum Message {
    /// Request to update the application UI
    Update,
    /// Change the application mode
    ChangeMode(CurrentMode),
    /// Change the focused component
    ChangeFocus(CurrentFocus),
    /// Select a workspace to view its tasks
    SelectWorkspace,
    /// Add a new item (workspace or task depending on context)
    AddItem,
    /// Add a child item (sub-workspace or sub-task)
    AddChild,
    /// Delete the currently selected item
    DeleteItem,
    /// Move selection up in the current component
    MoveUp,
    /// Move selection down in the current component
    MoveDown,
    /// Exit the application
    Exit,
    /// Archive the current workspace
    Archive,
    /// Recover an archived workspace
    Recovery,
    /// Mark the current task as completed
    Complete,
    /// Mark the current task as in process
    InProcess,
    /// Mark the current task as todo
    Todo,
    /// Mark the current task as deprecated
    Deprecated,
    /// Rename the currently selected item
    Rename,
    /// Filter tasks based on search criteria
    Filter,
    /// Handle search-related messages
    SearchMsg(SearchEvent),
    /// Show the help screen
    Help,
    /// Exit the help screen
    ExitHelp,
    /// Set due date for a task
    Due,
    /// Save application data to file
    SaveData,

    /// Increse task urgency
    IncreseUrgency,

    /// Decrese task urgency
    DecreseUrgency,

    /// Sort the task
    Sort,
}

/// State of which component is currently focused
///
/// This enum represents the three main components that can have focus
/// in the application UI. Only one component can be focused at a time,
/// and keyboard input is directed to the focused component.
///
/// # Variants
///
/// - `Workspace` - The main workspace list
/// - `TodoList` - The task list for the selected workspace
/// - `ArchivedWorkspace` - The list of archived workspaces
#[derive(Debug, Clone)]
pub enum CurrentFocus {
    /// Focus is on the main workspace list
    Workspace,
    /// Focus is on the todo list
    TodoList,
    /// Focus is on the archived workspace list
    ArchivedWorkspace,
}

/// State of which mode is currently active
///
/// This enum represents the different modes the application can be in,
/// which affects how keyboard input is interpreted and what actions are available.
///
/// # Variants
///
/// - `Normal` - Standard navigation mode
/// - `Insert` - Text input mode
/// - `Search` - Search/filter mode
/// - `Help` - Help screen display mode
#[derive(Debug, Clone, Copy)]
pub enum CurrentMode {
    /// Normal navigation mode where arrow keys move selection
    Normal,
    /// Text input mode for adding/editing items
    Insert,
    /// Search mode for filtering tasks
    Search,
    /// Help mode for displaying keybindings
    Help,
    /// Sort mode for displaying keybindings
    Sort,
}
