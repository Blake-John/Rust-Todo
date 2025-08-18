//! User Interface module
//!
//! This module contains all the UI components and related functionality for the
//! terminal-based todo application. It provides a complete TUI (Text User Interface)
//! implementation using the ratatui library.
//!
//! # Components
//!
//! The UI is composed of several key components:
//! - WorkspaceWidget: Displays and manages workspaces
//! - TodoWidget: Displays and manages tasks within workspaces
//! - CalendarWidget: Provides date selection functionality
//! - HelpWidget: Displays help information and keybindings
//! - PromptWidget: Shows status messages and current mode
//!
//! # Architecture
//!
//! The UI follows a message-passing architecture where:
//! 1. User input is captured and converted to UiMessage events
//! 2. Messages are processed by the handle_uimsg function
//! 3. UI state is updated accordingly
//! 4. The display is refreshed to reflect changes

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::vec;

use chrono::{Days, Local, Months, NaiveDate};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Clear, List, ListState, Padding, Paragraph};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
};
use regex::Regex;
use tokio::sync::mpsc;
use tui_textarea::TextArea;

use crate::app::appstate::{AppState, CurrentFocus, CurrentMode};
use crate::app::data::{self, Datas};
use crate::app::ui::calendarwidget::CalendarWidget;
use crate::app::ui::helpwidget::HelpWidget;
use crate::app::ui::prompt::PromptWidget;
use crate::app::ui::todolistwidget::{Task, TaskStatus, TodoList, TodoWidget};
use crate::app::ui::workspacewidget::Workspace;

pub mod calendarwidget;
pub mod helpwidget;
pub mod keymap;
pub mod prompt;
pub mod todolistwidget;
pub mod workspacewidget;
use workspacewidget::WorkspaceWidget;

/// UI Message for coordinating actions in the user interface
///
/// This enum represents all possible messages that can be sent to the UI
/// for processing. These messages drive the UI behavior and state changes
/// in response to user input and application events.
///
/// Messages are processed by the UI message handler and result in UI updates,
/// state changes, or interactions with other application components.
///
/// # Message Processing Flow
///
/// 1. User input is captured by the event handler
/// 2. Input is converted to appropriate UiMessage variants
/// 3. Messages are sent through a channel to the UI handler
/// 4. UI handler processes messages and updates UI state
/// 5. UI is re-rendered to reflect changes
///
/// # Variants
///
/// - `Update` - Request to update application data
/// - `UpdateUi` - Request to refresh/redraw the UI
/// - `SaveData` - Request to save application data to file
/// - `WAction(WidgetAction)` - Widget-specific action to perform
///
/// # Examples
///
/// ```
/// use crate::app::ui::{UiMessage, WidgetAction};
/// use crate::app::ui::todolistwidget::TaskStatus;
/// use crate::app::appstate::CurrentFocus;
///
/// // Example messages
/// let update_msg = UiMessage::Update;
/// let update_ui_msg = UiMessage::UpdateUi;
/// let save_msg = UiMessage::SaveData;
/// let widget_action_msg = UiMessage::WAction(WidgetAction::AddWorkspace);
/// ```
#[derive(Debug)]
pub enum UiMessage {
    /// Request to update application data
    Update,
    /// Request to refresh/redraw the UI
    UpdateUi,
    /// Request to save application data to file
    SaveData,
    /// Widget-specific action to perform
    WAction(WidgetAction),
}

/// Widget Action for changing widget states and performing operations
///
/// This enum represents specific actions that can be performed on UI widgets.
/// These actions correspond to user interactions like adding items, navigating,
/// deleting items, and changing states.
///
/// Widget actions are typically wrapped in UiMessage::WAction and processed
/// by the UI message handler to modify widget states and trigger UI updates.
///
/// # Action Categories
///
/// Widget actions can be grouped into several categories:
/// 1. Addition actions (AddWorkspace, AddTask, etc.)
/// 2. Navigation actions (SelectUp, SelectDown)
/// 3. Focus management (FocusWorkspace, etc.)
/// 4. Workspace navigation (EnterWorkspace, etc.)
/// 5. Deletion actions (DeleteWorkspace, DeleteTask)
/// 6. Task status changes (MarkTaskStatus)
/// 7. Workspace management (ArchiveWS, RecoveryWS)
/// 8. Item management (Rename, Filter)
/// 9. Help system (Help, ExitHelp)
/// 10. Date management (Due)
///
/// # Examples
///
/// ```
/// use crate::app::ui::WidgetAction;
/// use crate::app::ui::todolistwidget::TaskStatus;
/// use crate::app::appstate::CurrentFocus;
///
/// // Example widget actions
/// let add_workspace = WidgetAction::AddWorkspace;
/// let delete_task = WidgetAction::DeleteTask;
/// let mark_complete = WidgetAction::MarkTaskStatus(TaskStatus::Finished);
/// let rename = WidgetAction::Rename(CurrentFocus::Workspace);
/// ```
#[derive(Debug)]
pub enum WidgetAction {
    /// Add a new workspace at the root level
    AddWorkspace,
    /// Add a child workspace to the current workspace
    AddWorkspaceChild,
    /// Add a new task to the current todo list
    AddTask,
    /// Add a child task to the current task
    AddTaskChild,

    /// Move selection up in the current widget
    SelectUp,
    /// Move selection down in the current widget
    SelectDown,

    /// Focus on the main workspace widget
    FocusWorkspace,
    /// Focus on the todo list widget
    FocusTodolist,
    /// Focus on the archived workspace widget
    FocusArchivedWorkspace,

    /// Enter a workspace to view its tasks
    EnterWorkspace,
    /// Enter an archived workspace to view its tasks
    EnterArchivedWorkspace,

    /// Delete the currently selected workspace
    DeleteWorkspace,
    /// Delete the currently selected archived workspace
    DeleteArchivedWorkspace,
    /// Delete the currently selected task
    DeleteTask,

    /// Mark the current task with a specific status
    MarkTaskStatus(TaskStatus),
    /// Archive the current workspace
    ArchiveWS,
    /// Recover an archived workspace
    RecoveryWS,
    /// Rename the currently focused item
    Rename(CurrentFocus),
    /// Filter/search tasks
    Filter,
    /// Exit filter/search mode
    ExitFilter,

    /// Show the help screen
    Help,
    /// Exit the help screen
    ExitHelp,
    /// Set due date for the current task
    Due,
}

/// Selection direction for navigating lists
///
/// This enum is used to specify the direction of selection movement
/// when navigating through lists in the UI components.
///
/// # Variants
///
/// - `Back` - Move selection backward (up/left)
/// - `Forward` - Move selection forward (down/right)
///
/// # Examples
///
/// ```
/// use crate::app::ui::SelectBF;
///
/// // Example usage in a selection function
/// fn move_selection(direction: SelectBF) {
///     match direction {
///         SelectBF::Back => println!("Moving backward"),
///         SelectBF::Forward => println!("Moving forward"),
///     }
/// }
/// ```
#[derive(Debug)]
pub enum SelectBF {
    /// Move selection backward (up/left)
    Back,
    /// Move selection forward (down/right)
    Forward,
}

/// Search navigation events for filter operations
///
/// This enum represents the different navigation actions that can occur
/// during search/filter operations in the UI.
///
/// # Variants
///
/// - `Next` - Move to the next search result
/// - `Previous` - Move to the previous search result
/// - `Exit` - Exit the search mode
///
/// # Examples
///
/// ```
/// use crate::app::ui::SearchEvent;
///
/// // Example usage in a search handler
/// fn handle_search_event(event: SearchEvent) {
///     match event {
///         SearchEvent::Next => println!("Next search result"),
///         SearchEvent::Previous => println!("Previous search result"),
///         SearchEvent::Exit => println!("Exit search mode"),
///     }
/// }
/// ```
#[derive(Debug)]
pub enum SearchEvent {
    /// Move to the next search result
    Next,
    /// Move to the previous search result
    Previous,
    /// Exit the search mode
    Exit,
}

/// The Basic Structure of the UI
///
/// This struct represents the main UI component that orchestrates all the
/// individual widgets and manages their interactions. It handles rendering,
/// user input processing, and message handling for the entire application UI.
///
/// # Fields
///
/// - `workspace` ([`WorkspaceWidget`]) - The main workspace widget for displaying active workspaces
/// - `todolist` ([`TodoWidget`]) - The todo list widget for displaying tasks
/// - `archived_ws` ([`WorkspaceWidget`]) - The archived workspace widget for displaying archived workspaces
/// - `helpwidget` ([`HelpWidget`]) - The help widget for displaying keybindings and help information
/// - `prompt` ([`PromptWidget`]) - The prompt widget for displaying status messages
/// - `ui_rx` (`mpsc::Receiver<UiMessage>`) - Receiver for UI messages to process
/// - `input_rx` (`Arc<Mutex<mpsc::Receiver<KeyEvent>>>`) - Receiver for keyboard input events
///
/// # Examples
///
/// ```
/// use tokio::sync::mpsc;
/// use crossterm::event::KeyEvent;
/// use crate::app::ui::{Ui, UiMessage};
///
/// // Create channels for communication
/// let (ui_tx, ui_rx) = mpsc::channel(100);
/// let (input_tx, input_rx) = mpsc::channel(100);
///
/// // Create a new UI instance
/// let ui = Ui::new(ui_rx, input_rx);
/// ```
#[derive(Debug)]
pub struct Ui {
    /// The main workspace widget for displaying active workspaces
    pub workspace: WorkspaceWidget,
    /// The todo list widget for displaying tasks
    pub todolist: TodoWidget,
    /// The archived workspace widget for displaying archived workspaces
    pub archived_ws: WorkspaceWidget,
    /// The help widget for displaying keybindings and help information
    pub helpwidget: HelpWidget,
    /// The prompt widget for displaying status messages
    pub prompt: PromptWidget,
    /// Receiver for UI messages to process
    pub ui_rx: mpsc::Receiver<UiMessage>,
    /// Receiver for keyboard input events
    pub input_rx: Arc<Mutex<mpsc::Receiver<KeyEvent>>>,
}

pub trait SelectAction<T> {
    /// Select an item in a list, changing the current target to maintain consistency
    /// with the user's selection in the application.
    ///
    /// This function is used to navigate through lists of items (such as workspaces or tasks)
    /// and update the currently selected item based on the direction of movement.
    ///
    /// # Arguments
    ///
    /// - `bf` (`SelectBF`) - A [`SelectBF`] enum that determines whether to select backward or forward
    ///
    /// # Returns
    ///
    /// - `Option<Rc<RefCell<T>>>` - The result of the next selection, or None if no selection is possible
    fn get_selected_bf(
        &mut self,
        // current_target: &Option<Rc<RefCell<T>>>,
        // targets: &Vec<Rc<RefCell<T>>>,
        // state: &mut ListState,
        bf: SelectBF,
    ) -> Option<Rc<RefCell<T>>>;

    /// Get a flattened vector of T from a vector of [`T`] which might have nested [`T`] (children).
    ///
    /// This function recursively traverses a hierarchical structure of items (such as nested workspaces
    /// or tasks with subtasks) and returns a flat list of all items.
    ///
    /// # Arguments
    ///
    /// - `target` (`&Vec<Rc<RefCell<T>>>`) - The target vector to be flattened
    ///
    /// # Returns
    ///
    /// - `Vec<Rc<RefCell<T>>>` - The flattened vector containing all items from the hierarchy
    fn get_flattened(target: &Vec<Rc<RefCell<T>>>) -> Vec<Rc<RefCell<T>>>;
}

impl Ui {
    pub fn new(ui_rx: mpsc::Receiver<UiMessage>, input_rx: mpsc::Receiver<KeyEvent>) -> Self {
        Self {
            workspace: WorkspaceWidget::new(workspacewidget::WorkspaceType::Normal),
            todolist: TodoWidget::new(),
            archived_ws: WorkspaceWidget::new(workspacewidget::WorkspaceType::Archived),
            helpwidget: HelpWidget::new(),
            prompt: PromptWidget::new(),
            ui_rx,
            input_rx: Arc::new(Mutex::new(input_rx)),
        }
    }

    pub fn update(&mut self, f: &mut Frame) {
        let layout = Layout::vertical([Constraint::Fill(1), Constraint::Max(1)]).split(f.area());
        let layouts = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(layout[0]);
        let ws_layout = Layout::vertical([Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(layouts[0]);
        let utils_layout = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(self.prompt.desc.len() as u16 + 2),
        ])
        .split(layout[1]);

        f.render_widget(&mut self.workspace, ws_layout[0]);
        f.render_widget(&mut self.archived_ws, ws_layout[1]);
        f.render_widget(&mut self.todolist, layouts[1]);
        f.render_widget(&mut self.helpwidget.keymap, utils_layout[0]);
        f.render_widget(&mut self.prompt, utils_layout[1]);
        if let CurrentMode::Help = self.helpwidget.keymap.mode {
            f.render_widget(&mut self.helpwidget, f.area());
        }
    }
    pub async fn input_due_date(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<KeyEvent>>>,
        terminal: &mut DefaultTerminal,
        title: String,
        origin_due: Option<NaiveDate>,
    ) -> String {
        let mut textarea = TextArea::default();
        let placeholder = if let Some(due) = origin_due {
            due.to_string()
        } else {
            "".to_string()
        };
        textarea.set_placeholder_text(placeholder.clone());
        let mut item = String::new();
        let mut receiver = input_rx.lock().unwrap();
        let mut render_calendar = false;
        let mut calendar = CalendarWidget::new();
        loop {
            let _ = terminal.draw(|f| {
                self.prompt.desc = "In Insert Mode !".to_string();
                if render_calendar {
                    self.prompt.desc = "In Calendar Selection !".to_string();
                }
                self.update(f);
                // let area = Ui::get_popup_window_center(50, 20, f);
                let area = Ui::get_add_item_window(f);
                let block = Block::bordered()
                    .title(format!(" {} ", title))
                    .title_bottom(
                        Line::from(" press <ctrl-o> for calendar, input 'None' for unset ")
                            .right_aligned(),
                    );
                textarea.set_block(block);
                f.render_widget(Clear, area);
                f.render_widget(&textarea, area);
                if render_calendar {
                    f.render_widget(&mut calendar, f.area());
                }
            });
            if let Some(key_evt) = receiver.recv().await {
                if !render_calendar {
                    match key_evt.code {
                        KeyCode::Esc => break,
                        KeyCode::Backspace => {
                            textarea.delete_char();
                        }
                        KeyCode::Right => {
                            textarea.move_cursor(tui_textarea::CursorMove::Forward);
                        }
                        KeyCode::Left => {
                            textarea.move_cursor(tui_textarea::CursorMove::Back);
                        }
                        KeyCode::Enter => {
                            let content = textarea.into_lines();
                            content.iter().for_each(|s| {
                                item += s;
                            });
                            break;
                        }
                        KeyCode::Char('o') if key_evt.modifiers.contains(KeyModifiers::CONTROL) => {
                            render_calendar = true;
                        }
                        KeyCode::Char(c) => {
                            textarea.insert_char(c);
                        }
                        _ => {}
                    }
                } else {
                    match key_evt.code {
                        KeyCode::Char('h') | KeyCode::Left => {
                            calendar.move_left();
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            calendar.move_right();
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            calendar.move_down();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            calendar.move_up();
                        }
                        KeyCode::Esc => {
                            render_calendar = false;
                        }
                        KeyCode::Enter => {
                            item = calendar.cursor.to_string();
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
        drop(receiver);
        if item.is_empty() {
            placeholder
        } else if item == "None" {
            "".to_string()
        } else {
            item
        }
    }

    pub async fn get_input(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<KeyEvent>>>,
        terminal: &mut DefaultTerminal,
        title: String,
    ) -> String {
        let mut textarea = TextArea::default();
        let mut item = String::new();
        let mut receiver = input_rx.lock().unwrap();
        loop {
            let _ = terminal.draw(|f| {
                self.update(f);
                // let area = Ui::get_popup_window_center(50, 20, f);
                let area = Ui::get_add_item_window(f);
                let block = Block::bordered().title(format!(" {} ", title));
                textarea.set_block(block);
                f.render_widget(Clear, area);
                f.render_widget(&textarea, area);
            });
            if let Some(key_evt) = receiver.recv().await {
                match key_evt.code {
                    KeyCode::Esc => break,
                    KeyCode::Char(c) => {
                        textarea.insert_char(c);
                    }
                    KeyCode::Backspace => {
                        textarea.delete_char();
                    }
                    KeyCode::Right => {
                        textarea.move_cursor(tui_textarea::CursorMove::Forward);
                    }
                    KeyCode::Left => {
                        textarea.move_cursor(tui_textarea::CursorMove::Back);
                    }
                    KeyCode::Enter => {
                        let content = textarea.into_lines();
                        content.iter().for_each(|s| {
                            item += s;
                        });
                        break;
                    }

                    _ => {}
                }
            } else {
                break;
            }
        }
        drop(receiver);

        item
    }

    pub fn refresh_current(&mut self) {
        self.workspace.refresh_current();
        self.archived_ws.refresh_current();
        self.todolist
            .change_current_list(&self.workspace.current_workspace);
        self.helpwidget.keymap.focus = if self.archived_ws.focused {
            CurrentFocus::ArchivedWorkspace
        } else if self.todolist.focused {
            CurrentFocus::TodoList
        } else {
            CurrentFocus::Workspace
        };
    }

    pub async fn delete_item(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<KeyEvent>>>,
        terminal: &mut DefaultTerminal,
    ) -> bool {
        let _ = terminal.draw(|f| {
            // let area = Ui::get_popup_window_center(30, 10, f);
            let area = Ui::get_confirm_window(f);
            let block = Block::bordered().title(" Warn ").yellow();
            let info_line = Line::from(vec![
                "Do you want to ".into(),
                "Delete".red(),
                " this item ?".into(),
            ]);
            let confirm_line = Line::from(vec!["y/".red(), "n".yellow()]);
            let tip = Text::from(vec![info_line, confirm_line]).centered();
            let para = Paragraph::new(tip).centered().block(block).bold();
            self.update(f);
            f.render_widget(Clear, area);
            f.render_widget(para, area);
        });
        let mut receiver = input_rx.lock().unwrap();
        loop {
            if let Some(key_evt) = receiver.recv().await {
                match key_evt.code {
                    KeyCode::Char('y') => return true,
                    KeyCode::Char('n') | KeyCode::Esc => return false,
                    _ => {}
                }
            }
        }
    }

    pub async fn confirm_delete(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<KeyEvent>>>,
        terminal: &mut DefaultTerminal,
        target: CurrentFocus,
    ) -> bool {
        let _ = terminal.draw(|f| {
            // let area = Ui::get_popup_window_center(30, 10, f);
            let area = Ui::get_confirm_window(f);
            let block = Block::bordered().title(" Warn ").yellow();
            let info_line = match target {
                CurrentFocus::Workspace => Line::from(vec![
                    "The Current Workspace is ".into(),
                    "not empty ! ".red(),
                    "still delete ?".yellow(),
                ]),
                CurrentFocus::TodoList => Line::from(vec![
                    "The Todo List is ".into(),
                    "not empty ! ".red(),
                    "still delete ?".yellow(),
                ]),
                CurrentFocus::ArchivedWorkspace => Line::from(vec![
                    "The Archived Workspace is ".into(),
                    "has been archived ! ".red(),
                    "still delete ?".yellow(),
                ]),
            };
            let confirm_line = Line::from(vec!["y/".red(), "n".yellow()]);
            let tip = Text::from(vec![info_line, confirm_line]).centered();
            let para = Paragraph::new(tip).centered().block(block).bold();
            self.update(f);
            f.render_widget(Clear, area);
            f.render_widget(para, area);
        });
        let mut receiver = input_rx.lock().unwrap();
        loop {
            if let Some(key_evt) = receiver.recv().await {
                match key_evt.code {
                    KeyCode::Char('y') => return true,
                    KeyCode::Char('n') | KeyCode::Esc => return false,
                    _ => {}
                }
            }
        }
    }

    pub async fn filter_find(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<KeyEvent>>>,
        terminal: &mut DefaultTerminal,
    ) -> String {
        let mut textarea = TextArea::default();
        let mut item = String::new();
        let mut receiver = input_rx.lock().unwrap();
        loop {
            let _ = terminal.draw(|f| {
                self.update(f);

                let search_string = textarea.to_owned().into_lines();
                let mut tar_list = Vec::new();

                self.todolist
                    .current_todolist
                    .clone()
                    .unwrap()
                    .borrow()
                    .tasks
                    .iter()
                    .for_each(|task| {
                        if task.borrow().is_target(search_string.join(" ")) {
                            tar_list.push(task.to_owned());
                        }
                    });
                let tar_list_block = Block::bordered()
                    .title(" <3> Todo List ")
                    .border_style(Style::new().fg(Color::LightBlue))
                    .padding(Padding::uniform(1));
                let max_desc_len = TodoWidget::find_max_tasks_len(&tar_list, 1);
                let task_list = TodoWidget::get_search_list_item(
                    search_string.join(" "),
                    &tar_list,
                    1,
                    max_desc_len,
                );
                let tar_list_widget = List::new(task_list).block(tar_list_block);
                let layout =
                    Layout::vertical([Constraint::Fill(1), Constraint::Max(1)]).split(f.area());
                let tar_list_layout =
                    Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
                        .split(layout[0])[1];
                f.render_widget(Clear, tar_list_layout);
                f.render_widget(tar_list_widget, tar_list_layout);

                // let find_area = Ui::get_popup_window(30, 10, 45, 0, f);
                let find_area = Ui::get_filter_window(f);
                let filter_block = Block::bordered().title(" find ");
                textarea.set_block(filter_block);
                f.render_widget(Clear, find_area);
                f.render_widget(&textarea, find_area);
            });
            if let Some(key_evt) = receiver.recv().await {
                match key_evt.code {
                    KeyCode::Esc => break,
                    KeyCode::Char(c) => {
                        textarea.insert_char(c);
                    }
                    KeyCode::Backspace => {
                        textarea.delete_char();
                    }
                    KeyCode::Right => {
                        textarea.move_cursor(tui_textarea::CursorMove::Forward);
                    }
                    KeyCode::Left => {
                        textarea.move_cursor(tui_textarea::CursorMove::Back);
                    }
                    KeyCode::Enter => {
                        let content = textarea.into_lines();
                        content.iter().for_each(|s| {
                            item += s;
                        });
                        break;
                    }
                    _ => {}
                }
            }
        }
        drop(receiver);
        item
    }

    pub fn get_popup_window(
        percent_width: u16,
        percent_height: u16,
        x: u16,
        y: u16,
        f: &mut Frame,
    ) -> Rect {
        let v_layout = Layout::vertical([
            Constraint::Percentage(y),
            Constraint::Percentage(percent_height),
            Constraint::Fill(1),
        ])
        .split(f.area());
        let h_layout = Layout::horizontal([
            Constraint::Percentage(x),
            Constraint::Percentage(percent_width),
            Constraint::Fill(1),
        ])
        .split(v_layout[1]);
        h_layout[1]
    }

    pub fn get_filter_window(f: &mut Frame) -> Rect {
        let v_layout =
            Layout::vertical([Constraint::Min(3), Constraint::Percentage(100)]).split(f.area());

        let h_layout = Layout::horizontal([
            Constraint::Percentage(45),
            Constraint::Percentage(30),
            Constraint::Fill(1),
        ])
        .split(v_layout[0]);
        h_layout[1]
    }

    pub fn get_add_item_window(f: &mut Frame) -> Rect {
        let v_layout = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Min(3),
            Constraint::Percentage(50),
        ])
        .split(f.area());
        let h_layout = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Min(40),
            Constraint::Fill(1),
        ])
        .split(v_layout[1]);
        h_layout[1]
    }

    pub fn get_confirm_window(f: &mut Frame) -> Rect {
        let v_layout = Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Min(4),
            Constraint::Percentage(50),
        ])
        .split(f.area());
        let h_layout = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Min(40),
            Constraint::Fill(1),
        ])
        .split(v_layout[1]);
        h_layout[1]
    }

    pub fn get_popup_window_center_by_frame(percent_x: u16, percent_y: u16, f: &mut Frame) -> Rect {
        let layout1 = Layout::horizontal([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(f.area());

        Layout::vertical([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(layout1[1])[1]
    }

    pub fn get_popup_window_center_by_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let layout1 = Layout::horizontal([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(area);

        Layout::vertical([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(layout1[1])[1]
    }

    pub async fn handle_uimsg(
        &mut self,
        terminal: &mut DefaultTerminal,
        appstate: Arc<Mutex<AppState>>,
    ) {
        while let Some(msg) = self.ui_rx.recv().await {
            match msg {
                UiMessage::Update => {
                    let _result = terminal.draw(|f| self.update(f));
                }
                UiMessage::UpdateUi => {
                    let _result = terminal.draw(|f| self.update(f));
                }
                UiMessage::SaveData => {
                    let path = Path::new(
                        std::env::home_dir()
                            .unwrap_or(std::path::PathBuf::from("~"))
                            .as_path(),
                    )
                    .join(".todo/data.json");
                    let datas = Datas {
                        workspace: self.workspace.clone(),
                        todolist: self.todolist.clone(),
                        archived_ws: self.archived_ws.clone(),
                    };

                    let _ = data::save_data(path.as_path(), &datas);
                    self.prompt.desc = "Data Saved !".to_string();
                    let _ = terminal.draw(|f| self.update(f));
                }
                UiMessage::WAction(waction) => match waction {
                    WidgetAction::FocusWorkspace => {
                        self.workspace.focused = true;
                        self.todolist.focused = false;
                        self.archived_ws.focused = false;
                        self.helpwidget.keymap.focus = CurrentFocus::Workspace;
                        let _result = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::FocusTodolist => {
                        self.workspace.focused = false;
                        self.todolist.focused = true;
                        self.archived_ws.focused = false;
                        self.helpwidget.keymap.focus = CurrentFocus::TodoList;
                        let _result = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::FocusArchivedWorkspace => {
                        self.archived_ws.focused = true;
                        self.todolist.focused = false;
                        self.workspace.focused = false;
                        self.helpwidget.keymap.focus = CurrentFocus::ArchivedWorkspace;
                        let _result = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::AddWorkspace => {
                        let input_rx = self.input_rx.clone();
                        let result = self
                            .get_input(input_rx, terminal, "Add Workspace".to_string())
                            .await;
                        if !result.is_empty() {
                            let ws = Rc::new(RefCell::new(Workspace::new(result)));
                            let ws_id = ws.borrow().id;
                            self.workspace.add_workspace(ws);
                            self.todolist
                                .add_list(Rc::new(RefCell::new(TodoList::new(ws_id))));
                        }
                        self.prompt.desc = "Workspace Added !".to_string();
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::AddWorkspaceChild => {
                        let input_rx = self.input_rx.clone();
                        let result = self
                            .get_input(input_rx, terminal, "Add Subworkspace".to_string())
                            .await;
                        if !result.is_empty() {
                            let workspace = Rc::new(RefCell::new(Workspace::new(result)));
                            let ws_id = workspace.borrow().id.to_owned();
                            self.workspace.add_child_workspace(workspace);
                            self.todolist
                                .add_list(Rc::new(RefCell::new(TodoList::new(ws_id))));
                        }
                        self.prompt.desc = "Workspace Added !".to_string();
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::AddTask => {
                        let input_rx = self.input_rx.clone();
                        let result = self
                            .get_input(input_rx, terminal, "Add Task".to_string())
                            .await;
                        if !result.is_empty() {
                            if let Some(ctl) = &self.todolist.current_todolist {
                                let mut ctl_mut = ctl.borrow_mut();
                                ctl_mut.add_task(Rc::new(RefCell::new(Task::new(result, None))));
                            } else {
                                let ws =
                                    Rc::new(RefCell::new(Workspace::new("Workspace".to_string())));
                                let ws_id = ws.borrow().id;
                                let todolist = Rc::new(RefCell::new(TodoList::new(ws_id)));
                                todolist
                                    .borrow_mut()
                                    .add_task(Rc::new(RefCell::new(Task::new(result, None))));
                                self.workspace.add_workspace(ws.clone());
                                self.todolist.add_list(todolist.clone());
                                self.workspace.current_workspace = Some(ws);
                                self.todolist.current_todolist = Some(todolist);
                            }
                        }
                        self.prompt.desc = "Task Added !".to_string();
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::AddTaskChild => {
                        let input_rx = self.input_rx.clone();
                        let result = self
                            .get_input(input_rx, terminal, "Add Subtask".to_string())
                            .await;
                        if !result.is_empty()
                            && let Some(ctl) = &self.todolist.current_todolist
                        {
                            let mut ctl_mut = ctl.borrow_mut();
                            ctl_mut.add_child_task(Rc::new(RefCell::new(Task::new(result, None))));
                        }
                        self.prompt.desc = "Task Added !".to_string();
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::EnterWorkspace => {
                        let mut apps = appstate.lock().unwrap();
                        apps.current_focus = CurrentFocus::TodoList;
                        self.workspace.focused = false;
                        self.todolist.focused = true;
                        self.helpwidget.keymap.focus = CurrentFocus::TodoList;
                        self.todolist
                            .change_current_list(&self.workspace.current_workspace);
                        let _result = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::EnterArchivedWorkspace => {
                        let mut apps = appstate.lock().unwrap();
                        apps.current_focus = CurrentFocus::TodoList;
                        self.workspace.focused = false;
                        self.archived_ws.focused = false;
                        self.todolist.focused = true;
                        self.helpwidget.keymap.focus = CurrentFocus::TodoList;
                        self.todolist
                            .change_current_list(&self.archived_ws.current_workspace);
                        let _result = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::SelectUp => {
                        let apps = appstate.lock().unwrap();
                        if let CurrentMode::Help = apps.current_mode {
                            self.helpwidget.scroll = self.helpwidget.scroll.saturating_sub(1);
                            self.helpwidget.state =
                                self.helpwidget.state.position(self.helpwidget.scroll);
                        } else {
                            match apps.current_focus {
                                CurrentFocus::Workspace => {
                                    // self.workspace.current_workspace = Workspace::get_selected_bf(
                                    //     &self.workspace.current_workspace,
                                    //     &self.workspace.workspaces,
                                    //     &mut self.workspace.ws_state,
                                    //     SelectBF::Back,
                                    // );
                                    self.workspace.current_workspace =
                                        self.workspace.get_selected_bf(SelectBF::Back);
                                    self.todolist
                                        .change_current_list(&self.workspace.current_workspace);
                                }
                                CurrentFocus::TodoList => {
                                    let cur_task = self.todolist.get_selected_bf(SelectBF::Back);
                                    if let Some(cur_list) = &self.todolist.current_todolist {
                                        cur_list.borrow_mut().current_task = cur_task;
                                    }
                                    // if let Some(clist) = &self.todolist.current_todolist {
                                    //     let mut clist_mut = clist.borrow_mut();
                                    //     let tasks = clist_mut.tasks.clone();
                                    //     let ctask = clist_mut.current_task.clone();
                                    //     // let mut state = &mut clist.borrow_mut().state;
                                    //     // clist_mut.current_task = TodoList::get_selected_bf(
                                    //     //     &ctask,
                                    //     //     &tasks,
                                    //     //     &mut clist_mut.state,
                                    //     //     SelectBF::Back,
                                    //     // );
                                    // }
                                }
                                CurrentFocus::ArchivedWorkspace => {
                                    // self.archived_ws.current_workspace = Workspace::get_selected_bf(
                                    //     &self.archived_ws.current_workspace,
                                    //     &self.archived_ws.workspaces,
                                    //     &mut self.archived_ws.ws_state,
                                    //     SelectBF::Back,
                                    // );
                                    self.archived_ws.current_workspace =
                                        self.archived_ws.get_selected_bf(SelectBF::Back);
                                    self.todolist
                                        .change_current_list(&self.archived_ws.current_workspace);
                                }
                            }
                        }
                        let _ = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::SelectDown => {
                        let apps = appstate.lock().unwrap();
                        if let CurrentMode::Help = apps.current_mode {
                            self.helpwidget.scroll = self
                                .helpwidget
                                .scroll
                                .saturating_add(1)
                                .min(self.helpwidget.scroll_max);
                            self.helpwidget.state =
                                self.helpwidget.state.position(self.helpwidget.scroll);
                        } else {
                            match apps.current_focus {
                                CurrentFocus::Workspace => {
                                    // self.workspace.current_workspace = Workspace::get_selected_bf(
                                    //     &self.workspace.current_workspace,
                                    //     &self.workspace.workspaces,
                                    //     &mut self.workspace.ws_state,
                                    //     SelectBF::Forward,
                                    // );
                                    self.workspace.current_workspace =
                                        self.workspace.get_selected_bf(SelectBF::Forward);
                                    self.todolist
                                        .change_current_list(&self.workspace.current_workspace);
                                }
                                CurrentFocus::TodoList => {
                                    let cur_task = self.todolist.get_selected_bf(SelectBF::Forward);
                                    if let Some(cur_list) = &self.todolist.current_todolist {
                                        cur_list.borrow_mut().current_task = cur_task;
                                    }
                                    // if let Some(clist) = &self.todolist.current_todolist {
                                    //     let mut clist_mut = clist.borrow_mut();
                                    //     let tasks = clist_mut.tasks.clone();
                                    //     let ctask = clist_mut.current_task.clone();
                                    //     // let state = &mut clist_mut.state;
                                    //     clist_mut.current_task = TodoList::get_selected_bf(
                                    //         &ctask,
                                    //         &tasks,
                                    //         &mut clist_mut.state,
                                    //         SelectBF::Forward,
                                    //     );
                                    // }
                                }
                                CurrentFocus::ArchivedWorkspace => {
                                    // self.archived_ws.current_workspace = Workspace::get_selected_bf(
                                    //     &self.archived_ws.current_workspace,
                                    //     &self.archived_ws.workspaces,
                                    //     &mut self.archived_ws.ws_state,
                                    //     SelectBF::Forward,
                                    // );
                                    self.archived_ws.current_workspace =
                                        self.archived_ws.get_selected_bf(SelectBF::Forward);
                                    self.todolist
                                        .change_current_list(&self.archived_ws.current_workspace);
                                }
                            }
                        }
                        let _ = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::DeleteWorkspace => {
                        let input_rx = self.input_rx.clone();
                        let result = self.delete_item(input_rx, terminal).await;
                        if result {
                            let cur_ws_opt = self.workspace.current_workspace.clone();
                            let mut second_confirm = true;
                            if let Some(cur_ws) = &cur_ws_opt {
                                let cur_ws_bo = cur_ws.borrow();
                                if !cur_ws_bo.children.is_empty() {
                                    let input_rx = self.input_rx.clone();
                                    second_confirm = self
                                        .confirm_delete(input_rx, terminal, CurrentFocus::Workspace)
                                        .await;
                                }
                                if cur_ws_bo.has_todolist(&self.todolist) && second_confirm {
                                    let input_rx = self.input_rx.clone();
                                    second_confirm = self
                                        .confirm_delete(input_rx, terminal, CurrentFocus::TodoList)
                                        .await
                                }
                                if second_confirm {
                                    WorkspaceWidget::delete_item(
                                        &mut self.workspace.workspaces,
                                        cur_ws,
                                    );
                                    let tar_ws = cur_ws_bo.id;
                                    self.workspace.current_workspace = None;
                                    self.workspace.ws_state.select(None);
                                    self.todolist.delete_list(tar_ws);
                                }
                            }
                        }
                        self.prompt.desc = "Workspace Deleted !".to_string();
                        let _ = terminal.draw(|f| self.update(f));
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::DeleteArchivedWorkspace => {
                        let input_rx = self.input_rx.clone();
                        let result = self.delete_item(input_rx, terminal).await;
                        if result {
                            let cur_ws_opt = self.archived_ws.current_workspace.clone();
                            let mut second_confirm = true;
                            if let Some(cur_ws) = &cur_ws_opt {
                                let cur_ws_bo = cur_ws.borrow();
                                if !cur_ws_bo.children.is_empty() {
                                    let input_rx = self.input_rx.clone();
                                    second_confirm = self
                                        .confirm_delete(
                                            input_rx,
                                            terminal,
                                            CurrentFocus::ArchivedWorkspace,
                                        )
                                        .await;
                                }
                                if cur_ws_bo.has_todolist(&self.todolist) && second_confirm {
                                    let input_rx = self.input_rx.clone();
                                    second_confirm = self
                                        .confirm_delete(input_rx, terminal, CurrentFocus::TodoList)
                                        .await
                                }
                                if second_confirm {
                                    WorkspaceWidget::delete_item(
                                        &mut self.archived_ws.workspaces,
                                        cur_ws,
                                    );
                                    let tar_ws = cur_ws_bo.id;
                                    self.archived_ws.current_workspace = None;
                                    self.archived_ws.ws_state.select(None);
                                    self.todolist.delete_list(tar_ws);
                                }
                            }
                        }
                        self.prompt.desc = "Workspace Deleted !".to_string();
                        let _ = terminal.draw(|f| self.update(f));
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::DeleteTask => {
                        let input_rx = self.input_rx.clone();
                        let result = self.delete_item(input_rx, terminal).await;
                        if result {
                            let cur_list_opt = self.todolist.current_todolist.clone();
                            let mut to_second_confirm = false;
                            if let Some(cur_list) = cur_list_opt {
                                let cur_list = cur_list.borrow();
                                let cur_task_opt = cur_list.current_task.clone();
                                if let Some(cur_task) = cur_task_opt {
                                    let cur_task = cur_task.borrow();
                                    if !cur_task.children.is_empty() {
                                        to_second_confirm = true;
                                    }
                                }
                            }
                            if to_second_confirm {
                                let input_rx = self.input_rx.clone();
                                let second_confirm = self
                                    .confirm_delete(input_rx, terminal, CurrentFocus::TodoList)
                                    .await;
                                if second_confirm {
                                    let cur_list_opt = self.todolist.current_todolist.clone();
                                    if let Some(cur_list) = cur_list_opt {
                                        let mut cur_list_mut = cur_list.borrow_mut();
                                        cur_list_mut.delete_task();
                                    }
                                }
                            } else {
                                let cur_list_opt = self.todolist.current_todolist.clone();
                                if let Some(cur_list) = cur_list_opt {
                                    let mut cur_list_mut = cur_list.borrow_mut();
                                    cur_list_mut.delete_task();
                                }
                            }
                        }
                        self.prompt.desc = "Task Deleted !".to_string();
                        let _ = terminal.draw(|f| self.update(f));
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::MarkTaskStatus(status) => {
                        if let Some(cur_list) = &self.todolist.current_todolist
                            && let Some(cur_task) = &cur_list.borrow().current_task
                        {
                            Task::set_task_status(cur_task, status);
                        }
                        // if let Some(cur_list) = &self.todolist.current_todolist {
                        //     if let Some(cur_task) = &cur_list.borrow().current_task {
                        //         Task::set_task_status(cur_task, status);
                        //     }
                        // }
                        let _ = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::Rename(cur_focus) => {
                        match cur_focus {
                            CurrentFocus::Workspace => {
                                let cur_ws_opt = self.workspace.current_workspace.clone();
                                if let Some(cur_ws) = &cur_ws_opt {
                                    let input_rx = self.input_rx.clone();
                                    let new_name = self
                                        .get_input(input_rx, terminal, "Rename".to_string())
                                        .await;
                                    if !new_name.is_empty() {
                                        let mut cur_ws_mut = cur_ws.borrow_mut();
                                        cur_ws_mut.rename(new_name);
                                    }
                                }
                            }
                            CurrentFocus::TodoList => {
                                let mut can_renmae = false;
                                let cur_todolist_opt = self.todolist.current_todolist.clone();
                                if let Some(cur_todolist) = cur_todolist_opt {
                                    let cur_todolist_bor = cur_todolist.borrow();
                                    let cur_task_opt = cur_todolist_bor.current_task.clone();
                                    if cur_task_opt.is_some() {
                                        can_renmae = true;
                                    }
                                }

                                if can_renmae {
                                    let input_rx = self.input_rx.clone();
                                    let new_name = self
                                        .get_input(input_rx, terminal, "Rename".to_string())
                                        .await;
                                    if !new_name.is_empty() {
                                        let cur_list_opt = self.todolist.current_todolist.clone();
                                        if let Some(cur_list) = cur_list_opt {
                                            let cur_list_bor = cur_list.borrow();
                                            let cur_task_opt = cur_list_bor.current_task.clone();
                                            if let Some(cur_task) = cur_task_opt {
                                                let mut cur_task_mut = cur_task.borrow_mut();
                                                cur_task_mut.rename(new_name);
                                            }
                                        }
                                    }
                                }
                            }
                            CurrentFocus::ArchivedWorkspace => {
                                let cur_ws_opt = self.archived_ws.current_workspace.clone();
                                if let Some(cur_ws) = &cur_ws_opt {
                                    let input_rx = self.input_rx.clone();
                                    let new_name = self
                                        .get_input(input_rx, terminal, "Rename".to_string())
                                        .await;
                                    if !new_name.is_empty() {
                                        let mut cur_ws_mut = cur_ws.borrow_mut();
                                        cur_ws_mut.rename(new_name);
                                    }
                                }
                            }
                        }
                        let _ = terminal.draw(|f| self.update(f));
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::Filter => {
                        let cur_list_opt = self.todolist.current_todolist.clone();
                        if cur_list_opt.is_some() {
                            let input_rx = self.input_rx.clone();
                            let result = self.filter_find(input_rx, terminal).await;
                            self.todolist.search_string = result;
                            if let Some(cur_list) = &self.todolist.current_todolist {
                                let mut cur_list_mut = cur_list.borrow_mut();
                                cur_list_mut.state.select_first();
                                for task in cur_list_mut.tasks.iter() {
                                    if task.borrow().is_target(self.todolist.search_string.clone())
                                    {
                                        cur_list_mut.current_task = Some(task.to_owned());
                                        break;
                                    }
                                }
                            }
                        }
                        self.prompt.desc = "In Search Mode !".to_string();
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Search;
                    }
                    WidgetAction::ExitFilter => {
                        self.todolist.search_string = String::new();
                        if let Some(cur_list) = &self.todolist.current_todolist {
                            let mut cur_list_mut = cur_list.borrow_mut();
                            cur_list_mut.state = ListState::default();
                        }
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::ArchiveWS => {
                        let cur_ws_opt = self.workspace.current_workspace.clone();
                        if let Some(cur_ws) = &cur_ws_opt {
                            self.archived_ws.workspaces.push(cur_ws.to_owned());
                            WorkspaceWidget::delete_item(&mut self.workspace.workspaces, cur_ws);
                            self.workspace.current_workspace = None;
                            self.workspace.ws_state.select(None);
                        }
                        let _ = terminal.draw(|f| self.update(f));
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::RecoveryWS => {
                        let cur_ws_opt = self.archived_ws.current_workspace.clone();
                        if let Some(cur_ws) = &cur_ws_opt {
                            self.workspace.workspaces.push(cur_ws.to_owned());
                            WorkspaceWidget::delete_item(&mut self.archived_ws.workspaces, cur_ws);
                            self.archived_ws.current_workspace = None;
                            self.archived_ws.ws_state.select(None);
                        }
                        let _ = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::Help => {
                        self.helpwidget.keymap.mode = CurrentMode::Help;
                        self.prompt.desc = "In Help Mode !".to_string();
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                    }
                    WidgetAction::ExitHelp => {
                        self.helpwidget.keymap.mode = CurrentMode::Normal;
                        self.prompt.desc = "In Normal Mode !".to_string();
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                    }
                    WidgetAction::Due => {
                        let mut is_to_set = false;
                        let mut origin_due = None;
                        let mut apps = appstate.lock().unwrap();
                        let origin_mode = apps.current_mode;
                        apps.current_mode = CurrentMode::Insert;
                        drop(apps);

                        let cur_list_opt = self.todolist.current_todolist.clone();
                        if let Some(cur_list) = cur_list_opt {
                            let cur_task_opt = &cur_list.borrow().current_task;
                            if let Some(cur_task) = cur_task_opt {
                                is_to_set = true;
                                origin_due = cur_task.borrow().due;
                            }
                        }
                        if is_to_set {
                            let input_rx = self.input_rx.clone();
                            let date_str = self
                                .input_due_date(
                                    input_rx,
                                    terminal,
                                    "Set Due Date".to_string(),
                                    origin_due,
                                )
                                .await;
                            if let Some(cur_list) = &self.todolist.current_todolist {
                                let cur_task_opt = &cur_list.borrow().current_task;
                                if let Some(cur_task) = cur_task_opt {
                                    if date_str.is_empty() {
                                        cur_task.borrow_mut().due = None;
                                    } else {
                                        let date_result = NaiveDate::parse_from_str(
                                            date_str.as_str(),
                                            "%Y-%m-%d",
                                        );
                                        if let Ok(date) = date_result {
                                            cur_task.borrow_mut().due = Some(date);
                                        } else {
                                            let day_re = Regex::new(r"(\d+) days?").unwrap();
                                            let week_re = Regex::new(r"(\d+) weeks?").unwrap();
                                            let month_re = Regex::new(r"(\d+) months?").unwrap();

                                            if let Some(caped) =
                                                day_re.captures_at(date_str.as_str(), 0)
                                            {
                                                let date = Local::now()
                                                    .checked_add_days(Days::new(
                                                        caped[1].parse().unwrap_or_default(),
                                                    ))
                                                    .unwrap()
                                                    .date_naive();
                                                cur_task.borrow_mut().due = Some(date);
                                            } else if let Some(caped) =
                                                week_re.captures_at(date_str.as_str(), 0)
                                            {
                                                let day =
                                                    caped[1].parse::<i64>().unwrap_or_default() * 7;
                                                let date = Local::now()
                                                    .checked_add_days(Days::new(day as u64))
                                                    .unwrap()
                                                    .date_naive();
                                                cur_task.borrow_mut().due = Some(date);
                                            } else if let Some(caped) =
                                                month_re.captures_at(date_str.as_str(), 0)
                                            {
                                                let date = Local::now()
                                                    .checked_add_months(Months::new(
                                                        caped[1].parse().unwrap_or_default(),
                                                    ))
                                                    .unwrap()
                                                    .date_naive();
                                                cur_task.borrow_mut().due = Some(date);
                                            } else if date_str == "today" {
                                                cur_task.borrow_mut().due =
                                                    Some(Local::now().date_naive());
                                            } else if date_str == "tomorrow" {
                                                cur_task.borrow_mut().due = Some(
                                                    Local::now()
                                                        .checked_add_days(Days::new(1))
                                                        .unwrap()
                                                        .date_naive(),
                                                );
                                            } else {
                                                cur_task.borrow_mut().due =
                                                    Some(Local::now().date_naive());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = origin_mode;
                        self.prompt.desc = "Set Due Date !".to_string();
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                    }
                },
            }
        }
    }
}
