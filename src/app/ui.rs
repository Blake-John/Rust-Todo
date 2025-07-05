use std::cell::{Ref, RefCell};
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crossterm::event;
use futures::{FutureExt, StreamExt};
use ratatui::layout::Rect;
use ratatui::text;
use ratatui::widgets::Block;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
};
use tokio::sync::mpsc;
use tui_textarea::{Input, Key, TextArea};

use crate::app::appstate::{AppState, CurrentFocus, CurrentMode};
use crate::app::ui::workspacewidget::Workspace;

mod workspacewidget;
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
    AddTodoList,
    AddWorkspaceChild,
    SelectUp,
    SelectDown,
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
    pub todolist: (),
    pub ui_rx: mpsc::Receiver<UiMessage>,
    pub input_rx: Arc<Mutex<mpsc::Receiver<InputEvent>>>,
}

impl Ui {
    pub fn new(ui_rx: mpsc::Receiver<UiMessage>, input_rx: mpsc::Receiver<InputEvent>) -> Self {
        Self {
            workspace: WorkspaceWidget::new(),
            todolist: (),
            ui_rx,
            input_rx: Arc::new(Mutex::new(input_rx)),
        }
    }

    pub fn update(&mut self, f: &mut Frame) {
        let layouts = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(f.area());

        f.render_widget(&mut self.workspace, layouts[0]);
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
                        textarea.move_cursor(tui_textarea::CursorMove::Back);
                    }
                    InputEvent::Left => {
                        textarea.move_cursor(tui_textarea::CursorMove::Forward);
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

        item
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

    pub fn get_flattened(workspaces: &Vec<Rc<RefCell<Workspace>>>) -> Vec<Rc<RefCell<Workspace>>> {
        let mut result = Vec::<Rc<RefCell<Workspace>>>::new();
        workspaces.iter().for_each(|ws| {
            result.push(ws.clone());
            let ws_ = ws.borrow();
            if !ws_.children.is_empty() {
                let child = Ui::get_flattened(&ws_.children);
                result.extend(child);
            }
        });

        result
    }

    pub fn get_selected_bf(
        current_ws: &Option<Rc<RefCell<Workspace>>>,
        workspaces: &Vec<Rc<RefCell<Workspace>>>,
        bf: SelectBF,
    ) -> Option<Rc<RefCell<Workspace>>> {
        let ws_list = Ui::get_flattened(workspaces);
        if workspaces.len() > 0 {
            if current_ws.is_none() {
                Some(ws_list[0].clone())
            } else {
                let mut target = 0;

                if let Some(cw) = current_ws {
                    let (i, _) = ws_list
                        .iter()
                        .enumerate()
                        .find(|(i, ws)| ws.borrow().desc == cw.borrow().desc)
                        .unwrap();
                    target = i;
                }
                match bf {
                    SelectBF::Back => {
                        if target != 0 {
                            target -= 1;
                        }
                    }
                    SelectBF::Forward => {
                        if target < ws_list.len() - 1 {
                            target += 1;
                        }
                    }
                }

                Some(ws_list[target].clone())
            }
        } else {
            None
        }
    }

    pub async fn handle_uimsg(
        &mut self,
        terminal: &mut DefaultTerminal,
        appstate: Arc<Mutex<AppState>>,
    ) {
        loop {
            if let Some(msg) = self.ui_rx.recv().await {
                match msg {
                    UiMessage::Update => {
                        println!("");
                    }
                    UiMessage::UpdateUi => {
                        let _result = terminal.draw(|f| self.update(f));
                    }
                    UiMessage::WAction(waction) => match waction {
                        WidgetAction::AddWorkspace => {
                            let input_rx = self.input_rx.clone();
                            let result = self.add_item(input_rx, terminal).await;
                            if !result.is_empty() {
                                self.workspace
                                    .workspaces
                                    .push(Rc::new(RefCell::new(Workspace::new(result))));
                            }
                            let _ = terminal.draw(|f| {
                                self.update(f);
                            });
                            let mut apps = appstate.lock().unwrap();
                            apps.current_mode = CurrentMode::Normal;
                        }
                        WidgetAction::AddTodoList => {
                            let input_rx = self.input_rx.clone();
                            let _result = self.add_item(input_rx, terminal);
                        }
                        WidgetAction::AddWorkspaceChild => {
                            let input_rx = self.input_rx.clone();
                            let result = self.add_item(input_rx, terminal).await;
                            if !result.is_empty() {
                                if let Some(cw) = &self.workspace.current_workspace {
                                    let mut cw_mut = cw.borrow_mut();
                                    cw_mut.add_child(Rc::new(RefCell::new(Workspace::new(result))));
                                } else {
                                    self.workspace
                                        .workspaces
                                        .push(Rc::new(RefCell::new(Workspace::new(result))));
                                }
                            }
                            let _ = terminal.draw(|f| {
                                self.update(f);
                            });
                            let mut apps = appstate.lock().unwrap();
                            apps.current_mode = CurrentMode::Normal;
                        }
                        WidgetAction::SelectUp => {
                            let apps = appstate.lock().unwrap();
                            match apps.current_focus {
                                CurrentFocus::Workspace => {
                                    self.workspace.current_workspace = Ui::get_selected_bf(
                                        &self.workspace.current_workspace,
                                        &self.workspace.workspaces,
                                        SelectBF::Back,
                                    );
                                    let _ = terminal.draw(|f| self.update(f));
                                }
                                _ => {}
                            }
                        }
                        WidgetAction::SelectDown => {
                            let apps = appstate.lock().unwrap();
                            match apps.current_focus {
                                CurrentFocus::Workspace => {
                                    self.workspace.current_workspace = Ui::get_selected_bf(
                                        &self.workspace.current_workspace,
                                        &self.workspace.workspaces,
                                        SelectBF::Forward,
                                    );
                                    let _ = terminal.draw(|f| self.update(f));
                                }
                                _ => {}
                            }
                        }
                    },
                }
            } else {
                break;
            }
        }
    }
}
