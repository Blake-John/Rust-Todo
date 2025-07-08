use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::vec;

use ratatui::layout::{Alignment, Rect};
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
use crate::app::ui::todolistwidget::{Task, TodoList, TodoWidget};
use crate::app::ui::workspacewidget::Workspace;

pub mod todolistwidget;
pub mod workspacewidget;
use workspacewidget::WorkspaceWidget;

#[derive(Debug)]
pub enum UiMessage {
    Update,
    UpdateUi,
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
    EnterWorkspace,
    DeleteWorkspace,
    DeleteTask,
}

#[derive(Debug)]
pub enum SelectBF {
    Back,
    Forward,
}

#[derive(Debug)]
pub enum InputEvent {
    InsertChar(char),
    Backspace,
    Left,
    Right,
    Enter,
    Esc,
}

#[derive(Debug)]
pub struct Ui {
    pub workspace: WorkspaceWidget,
    pub todolist: TodoWidget,
    pub ui_rx: mpsc::Receiver<UiMessage>,
    pub input_rx: Arc<Mutex<mpsc::Receiver<InputEvent>>>,
}

pub trait SelectAction<T> {
    fn get_selected_bf(
        current_target: &Option<Rc<RefCell<T>>>,
        targets: &Vec<Rc<RefCell<T>>>,
        state: &mut ListState,
        bf: SelectBF,
    ) -> Option<Rc<RefCell<T>>>;
    fn get_flattened(target: &Vec<Rc<RefCell<T>>>) -> Vec<Rc<RefCell<T>>>;
}

impl Ui {
    pub fn new(ui_rx: mpsc::Receiver<UiMessage>, input_rx: mpsc::Receiver<InputEvent>) -> Self {
        Self {
            workspace: WorkspaceWidget::new(),
            todolist: TodoWidget::new(),
            ui_rx,
            input_rx: Arc::new(Mutex::new(input_rx)),
        }
    }

    pub fn update(&mut self, f: &mut Frame) {
        let layouts = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(f.area());

        f.render_widget(&mut self.workspace, layouts[0]);
        f.render_widget(&mut self.todolist, layouts[1]);
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
                let area = Ui::get_popup_window(50, 20, f);
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

    pub async fn delete_item(
        &mut self,
        input_rx: Arc<Mutex<mpsc::Receiver<InputEvent>>>,
        terminal: &mut DefaultTerminal,
    ) -> bool {
        let _ = terminal.draw(|f| {
            let area = Ui::get_popup_window(30, 10, f);
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

    pub fn get_popup_window(percent_x: u16, percent_y: u16, f: &mut Frame) -> Rect {
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
                        let _result = terminal.draw(|f| self.update(f));
                    }
                    WidgetAction::FocusTodolist => {
                        self.workspace.focused = false;
                        self.todolist.focused = true;
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
                        self.todolist
                            .change_current_list(&self.workspace.current_workspace);
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
                        }
                    }
                    WidgetAction::DeleteWorkspace => {
                        let input_rx = self.input_rx.clone();
                        let result = self.delete_item(input_rx, terminal).await;
                        if result {
                            WorkspaceWidget::delete_item(
                                &mut self.workspace.workspaces,
                                &self.workspace.current_workspace,
                            );
                            let tar_ws = self
                                .workspace
                                .current_workspace
                                .clone()
                                .unwrap()
                                .borrow_mut()
                                .id;
                            self.workspace.current_workspace = None;
                            self.todolist.delete_list(tar_ws);
                        }
                        let _ = terminal.draw(|f| self.update(f));
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                    WidgetAction::DeleteTask => {
                        let input_rx = self.input_rx.clone();
                        let result = self.delete_item(input_rx, terminal).await;
                        if result {
                            // let ctl = &self.todolist.current_todolist;
                            if let Some(cur_list) = &self.todolist.current_todolist {
                                let cur_list_ = cur_list.clone();
                                let ctask = cur_list_.borrow().current_task.clone();
                                if let Some(cur_task) = ctask {
                                    TodoWidget::delete_task(
                                        &mut self.todolist.todolists,
                                        cur_list,
                                        &cur_task,
                                    );
                                    self.todolist
                                        .change_current_list(&self.workspace.current_workspace);
                                }
                            }
                        }
                        let _ = terminal.draw(|f| self.update(f));
                        let mut apps = appstate.lock().unwrap();
                        apps.current_mode = CurrentMode::Normal;
                    }
                },
            }
        }
    }
}
