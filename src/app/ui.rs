use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::vec;

use keymap::KeyMap;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Clear, ListState, Paragraph};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
};
use tokio::sync::mpsc;
use tui_textarea::TextArea;

use crate::app::appstate::{AppState, CurrentFocus, CurrentMode};
use crate::app::ui::todolistwidget::{Task, TaskStatus, TodoList, TodoWidget};
use crate::app::ui::workspacewidget::Workspace;

pub mod keymap;
pub mod todolistwidget;
pub mod workspacewidget;
use workspacewidget::WorkspaceWidget;

/// UiMessage to perform actions
///
/// # Variants
///
/// - `Update` - update the data
/// - `UpdateUi` - update the ui
/// - `WAction(WidgetAction)` - the action of the widget
#[derive(Debug)]
pub enum UiMessage {
    Update,
    UpdateUi,
    WAction(WidgetAction),
}

/// WidgetAction to change the widget state
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
}

/// The select direction, whether to go back or forward
#[derive(Debug)]
pub enum SelectBF {
    Back,
    Forward,
}

/// the input event to define the input action
#[derive(Debug)]
pub enum InputEvent {
    InsertChar(char),
    Backspace,
    Left,
    Right,
    Enter,
    Esc,
}

/// The Basic Structure of the UI
///
/// # Fields
///
/// - `workspace` ([`WorkspaceWidget`]) - a widget to display the workspace
/// - `todolist` ([`TodoWidget`]) - a widget to display the todo list
/// - `ui_rx` (`mpsc`) - a mpsc receiver to receive [`UiMessage`]
/// - `input_rx` (`Arc<Mutex<mpsc::Receiver<InputEvent>>>`) - a mpsc receiver to receive [`InputEvent`]
#[derive(Debug)]
pub struct Ui {
    pub workspace: WorkspaceWidget,
    pub todolist: TodoWidget,
    pub archived_ws: WorkspaceWidget,
    pub keymap: KeyMap,
    pub ui_rx: mpsc::Receiver<UiMessage>,
    pub input_rx: Arc<Mutex<mpsc::Receiver<InputEvent>>>,
}

pub trait SelectAction<T> {
    /// a function to select an item, which is used to change the current target of [`T`]
    /// inorder to make it consistent with what you selected in the application
    ///
    /// # Arguments
    ///
    /// - `current_target` (`&Option<Rc<RefCell<T>>>`) - what you are currently selecting
    /// - `targets` (`&Vec<Rc<RefCell<T>>>`) - from which list to change the selection
    /// - `state` (`&mut ListState`) - a [`ListState`] of [`List`] to show the selection in the ui
    /// - `bf` (`SelectBF`) - a [`SelectBF`] enum determines whether to select backward or forward
    ///
    /// # Returns
    ///
    /// - `Option<Rc<RefCell<T>>>` - the result of the next selection
    fn get_selected_bf(
        current_target: &Option<Rc<RefCell<T>>>,
        targets: &Vec<Rc<RefCell<T>>>,
        state: &mut ListState,
        bf: SelectBF,
    ) -> Option<Rc<RefCell<T>>>;
    /// Get the flattened vector of T from the vector of [`T`] which might have nested [`T`] (children)
    ///
    /// # Arguments
    ///
    /// - `target` (`&Vec<Rc<RefCell<T>>>`) - target to be get flattened
    ///
    /// # Returns
    ///
    /// - `Vec<Rc<RefCell<T>>>` - the flattened vector of the target
    fn get_flattened(target: &Vec<Rc<RefCell<T>>>) -> Vec<Rc<RefCell<T>>>;
}

impl Ui {
    pub fn new(ui_rx: mpsc::Receiver<UiMessage>, input_rx: mpsc::Receiver<InputEvent>) -> Self {
        Self {
            workspace: WorkspaceWidget::new(workspacewidget::WorkspaceType::Normal),
            todolist: TodoWidget::new(),
            archived_ws: WorkspaceWidget::new(workspacewidget::WorkspaceType::Archived),
            keymap: KeyMap::default(),
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

        f.render_widget(&mut self.workspace, ws_layout[0]);
        f.render_widget(&mut self.archived_ws, ws_layout[1]);
        f.render_widget(&mut self.todolist, layouts[1]);
        f.render_widget(&mut self.keymap, layout[1]);
    }

    pub async fn add_item(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<InputEvent>>>,
        terminal: &mut DefaultTerminal,
    ) -> String {
        let mut textarea = TextArea::default();
        let mut item = String::new();
        let mut receiver = input_rx.lock().unwrap();
        loop {
            let _ = terminal.draw(|f| {
                self.update(f);
                let area = Ui::get_popup_window_center(50, 20, f);
                let block = Block::bordered().title(" Add Item ");
                textarea.set_block(block);
                f.render_widget(Clear, area);
                f.render_widget(&textarea, area);
            });
            if let Some(evt) = receiver.recv().await {
                match evt {
                    InputEvent::Esc => break,
                    InputEvent::InsertChar(c) => {
                        textarea.insert_char(c);
                    }
                    InputEvent::Backspace => {
                        textarea.delete_char();
                    }
                    InputEvent::Right => {
                        textarea.move_cursor(tui_textarea::CursorMove::Forward);
                    }
                    InputEvent::Left => {
                        textarea.move_cursor(tui_textarea::CursorMove::Back);
                    }
                    InputEvent::Enter => {
                        let content = textarea.to_owned().into_lines();
                        content.iter().for_each(|s| {
                            item += s;
                        });
                        break;
                    }
                }
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
        self.keymap.focus = if self.archived_ws.focused {
            CurrentFocus::ArchivedWorkspace
        } else if self.todolist.focused {
            CurrentFocus::TodoList
        } else {
            CurrentFocus::Workspace
        };
    }

    pub async fn delete_item(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<InputEvent>>>,
        terminal: &mut DefaultTerminal,
    ) -> bool {
        let _ = terminal.draw(|f| {
            let area = Ui::get_popup_window_center(30, 10, f);
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
            if let Some(evt) = receiver.recv().await {
                match evt {
                    InputEvent::InsertChar('y') => {
                        return true;
                    }
                    InputEvent::InsertChar('n') => {
                        return false;
                    }
                    InputEvent::Esc => {
                        return false;
                    }
                    _ => {}
                }
            }
        }
    }

    pub async fn confirm_delete(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<InputEvent>>>,
        terminal: &mut DefaultTerminal,
        target: CurrentFocus,
    ) -> bool {
        let _ = terminal.draw(|f| {
            let area = Ui::get_popup_window_center(30, 10, f);
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
            if let Some(evt) = receiver.recv().await {
                match evt {
                    InputEvent::InsertChar('y') => {
                        return true;
                    }
                    InputEvent::InsertChar('n') => {
                        return false;
                    }
                    InputEvent::Esc => {
                        return false;
                    }
                    _ => {}
                }
            }
        }
    }

    pub async fn filter_find(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<InputEvent>>>,
        terminal: &mut DefaultTerminal,
    ) {
        let mut textarea = TextArea::default();
        let mut item = String::new();
        let mut receiver = input_rx.lock().unwrap();
        loop {
            let _ = terminal.draw(|f| {
                self.update(f);
                let area = Ui::get_popup_window(30, 12, 45, 1, f);
                let filter_block = Block::bordered().title(" find ");
                textarea.set_block(filter_block);
                f.render_widget(Clear, area);
                f.render_widget(&textarea, area);
            });
            if let Some(evt) = receiver.recv().await {
                match evt {
                    InputEvent::Esc => break,
                    InputEvent::InsertChar(c) => {
                        textarea.insert_char(c);
                    }
                    InputEvent::Backspace => {
                        textarea.delete_char();
                    }
                    InputEvent::Right => {
                        textarea.move_cursor(tui_textarea::CursorMove::Forward);
                    }
                    InputEvent::Left => {
                        textarea.move_cursor(tui_textarea::CursorMove::Back);
                    }
                    InputEvent::Enter => {
                        let content = textarea.to_owned().into_lines();
                        content.iter().for_each(|s| {
                            item += s;
                        });
                        break;
                    }
                }
            }
        }
        drop(receiver);
    }

    pub fn get_popup_window(
        percent_width: u16,
        percent_height: u16,
        x: u16,
        y: u16,
        f: &mut Frame,
    ) -> Rect {
        let v_leyout = Layout::vertical([
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
        .split(v_leyout[1]);
        h_layout[1]
    }

    pub fn get_popup_window_center(percent_x: u16, percent_y: u16, f: &mut Frame) -> Rect {
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
                UiMessage::WAction(waction) => match waction {
                    WidgetAction::FocusWorkspace => {
                        self.workspace.focused = true;
                        self.todolist.focused = false;
                        self.archived_ws.focused = false;
                        self.keymap.focus = CurrentFocus::Workspace;
                        let _result = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::FocusTodolist => {
                        self.workspace.focused = false;
                        self.todolist.focused = true;
                        self.archived_ws.focused = false;
                        self.keymap.focus = CurrentFocus::TodoList;
                        let _result = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::FocusArchivedWorkspace => {
                        self.archived_ws.focused = true;
                        self.todolist.focused = false;
                        self.workspace.focused = false;
                        self.keymap.focus = CurrentFocus::ArchivedWorkspace;
                        let _result = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::AddWorkspace => {
                        let input_rx = self.input_rx.clone();
                        let result = self.add_item(input_rx, terminal).await;
                        if !result.is_empty() {
                            let ws = Rc::new(RefCell::new(Workspace::new(result)));
                            let ws_id = ws.borrow().id;
                            self.workspace.add_workspace(ws);
                            self.todolist
                                .add_list(Rc::new(RefCell::new(TodoList::new(ws_id))));
                        }
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::AddWorkspaceChild => {
                        let input_rx = self.input_rx.clone();
                        let result = self.add_item(input_rx, terminal).await;
                        if !result.is_empty() {
                            let workspace = Rc::new(RefCell::new(Workspace::new(result)));
                            let ws_id = workspace.borrow().id.to_owned();
                            self.workspace.add_child_workspace(workspace);
                            self.todolist
                                .add_list(Rc::new(RefCell::new(TodoList::new(ws_id))));
                        }
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::AddTask => {
                        let input_rx = self.input_rx.clone();
                        let result = self.add_item(input_rx, terminal).await;
                        if !result.is_empty() {
                            if let Some(ctl) = &self.todolist.current_todolist {
                                let mut ctl_mut = ctl.borrow_mut();
                                ctl_mut.add_task(Rc::new(RefCell::new(Task::new(result, None))));
                            }
                        }
                        let _ = terminal.draw(|f| {
                            self.update(f);
                        });
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::AddTaskChild => {
                        let input_rx = self.input_rx.clone();
                        let result = self.add_item(input_rx, terminal).await;
                        if !result.is_empty() {
                            if let Some(ctl) = &self.todolist.current_todolist {
                                let mut ctl_mut = ctl.borrow_mut();
                                ctl_mut
                                    .add_child_task(Rc::new(RefCell::new(Task::new(result, None))));
                            }
                        }
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
                        self.keymap.focus = CurrentFocus::TodoList;
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
                        self.keymap.focus = CurrentFocus::TodoList;
                        self.todolist
                            .change_current_list(&self.archived_ws.current_workspace);
                        let _result = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::SelectUp => {
                        let apps = appstate.lock().unwrap();
                        match apps.current_focus {
                            CurrentFocus::Workspace => {
                                self.workspace.current_workspace = Workspace::get_selected_bf(
                                    &self.workspace.current_workspace,
                                    &self.workspace.workspaces,
                                    &mut self.workspace.ws_state,
                                    SelectBF::Back,
                                );
                                self.todolist
                                    .change_current_list(&self.workspace.current_workspace);
                                let _ = terminal.draw(|f| self.update(f));
                            }
                            CurrentFocus::TodoList => {
                                if let Some(clist) = &self.todolist.current_todolist {
                                    let mut clist_mut = clist.borrow_mut();
                                    let tasks = clist_mut.tasks.clone();
                                    let ctask = clist_mut.current_task.clone();
                                    // let mut state = &mut clist.borrow_mut().state;
                                    clist_mut.current_task = TodoList::get_selected_bf(
                                        &ctask,
                                        &tasks,
                                        &mut clist_mut.state,
                                        SelectBF::Back,
                                    );
                                }

                                let _ = terminal.draw(|f| self.update(f));
                            }
                            CurrentFocus::ArchivedWorkspace => {
                                self.archived_ws.current_workspace = Workspace::get_selected_bf(
                                    &self.archived_ws.current_workspace,
                                    &self.archived_ws.workspaces,
                                    &mut self.archived_ws.ws_state,
                                    SelectBF::Back,
                                );
                                self.todolist
                                    .change_current_list(&self.archived_ws.current_workspace);
                                let _ = terminal.draw(|f| self.update(f));
                            }
                        }
                    }
                    WidgetAction::SelectDown => {
                        let apps = appstate.lock().unwrap();
                        match apps.current_focus {
                            CurrentFocus::Workspace => {
                                self.workspace.current_workspace = Workspace::get_selected_bf(
                                    &self.workspace.current_workspace,
                                    &self.workspace.workspaces,
                                    &mut self.workspace.ws_state,
                                    SelectBF::Forward,
                                );
                                self.todolist
                                    .change_current_list(&self.workspace.current_workspace);
                                let _ = terminal.draw(|f| self.update(f));
                            }
                            CurrentFocus::TodoList => {
                                if let Some(clist) = &self.todolist.current_todolist {
                                    let mut clist_mut = clist.borrow_mut();
                                    let tasks = clist_mut.tasks.clone();
                                    let ctask = clist_mut.current_task.clone();
                                    // let state = &mut clist_mut.state;
                                    clist_mut.current_task = TodoList::get_selected_bf(
                                        &ctask,
                                        &tasks,
                                        &mut clist_mut.state,
                                        SelectBF::Forward,
                                    );
                                }

                                let _ = terminal.draw(|f| self.update(f));
                            }
                            CurrentFocus::ArchivedWorkspace => {
                                self.archived_ws.current_workspace = Workspace::get_selected_bf(
                                    &self.archived_ws.current_workspace,
                                    &self.archived_ws.workspaces,
                                    &mut self.archived_ws.ws_state,
                                    SelectBF::Forward,
                                );
                                self.todolist
                                    .change_current_list(&self.archived_ws.current_workspace);
                                let _ = terminal.draw(|f| self.update(f));
                            }
                        }
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
                            }
                        }
                        let _ = terminal.draw(|f| self.update(f));
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::MarkTaskStatus(status) => {
                        if let Some(cur_list) = &self.todolist.current_todolist {
                            if let Some(cur_task) = &cur_list.borrow().current_task {
                                Task::set_task_status(cur_task, status);
                            }
                        }
                        let _ = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::Rename(cur_focus) => {
                        match cur_focus {
                            CurrentFocus::Workspace => {
                                let cur_ws_opt = self.workspace.current_workspace.clone();
                                if let Some(cur_ws) = &cur_ws_opt {
                                    let input_rx = self.input_rx.clone();
                                    let new_name = self.add_item(input_rx, terminal).await;
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
                                    let new_name = self.add_item(input_rx, terminal).await;
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
                                    let new_name = self.add_item(input_rx, terminal).await;
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
                    // TODO: Implement the filter functionality
                    WidgetAction::Filter => {
                        let cur_list_opt = self.todolist.current_todolist.clone();
                        if let Some(cur_list) = cur_list_opt {
                            let input_rx = self.input_rx.clone();
                            self.filter_find(input_rx, terminal).await;
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
                    }
                    _ => {}
                },
            }
        }
    }
}
