use std::sync::{Arc, Mutex};

use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::Stylize,
    widgets::{Block, List, ListItem, StatefulWidget, Widget},
};
use tokio::sync::mpsc;

use crate::app::appstate::AppState;

#[derive(Debug)]
pub enum UiMessage {
    Update,
    UpdateUi,
}

#[derive(Debug)]
pub struct Ui {
    pub workspace: WorkspaceWidget,
    pub todolist: (),
    pub ui_rx: mpsc::Receiver<UiMessage>,
}

impl Ui {
    pub fn new(ui_rx: mpsc::Receiver<UiMessage>) -> Self {
        Self {
            workspace: WorkspaceWidget::new(),
            todolist: (),
            ui_rx,
        }
    }

    pub fn update(&mut self, f: &mut Frame) {
        let layouts = Layout::horizontal([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(f.area());

        f.render_widget(&mut self.workspace, layouts[0]);
    }

    pub async fn hanlde_uimsg(
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
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub desc: String,
    pub selected: bool,
    pub children: Vec<Arc<Workspace>>,
}

impl Workspace {
    fn add_child(&mut self, workspace: Arc<Self>) {
        self.children.push(workspace);
    }

    fn add_children(&mut self, workspaces: Vec<Arc<Self>>) {
        workspaces.iter().for_each(|workspace| {
            self.add_child(workspace.clone());
        });
    }
}

#[derive(Debug)]
struct WorkspaceWidget {
    workspaces: Vec<Arc<Workspace>>,
    current_workspace: Option<Arc<Workspace>>,
}

impl WorkspaceWidget {
    fn new() -> Self {
        Self {
            workspaces: Vec::<Arc<Workspace>>::new(),
            current_workspace: None,
        }
    }
}

impl Widget for &mut WorkspaceWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let mut workspace_list = Vec::<ListItem>::new();
        self.workspaces.iter().for_each(|workspace| {
            workspace_list.push(ListItem::new(workspace.desc.clone().green()));
        });

        let workspace_block = Block::bordered().title(" Workspace ".green()).green();

        let list_widget = List::new(workspace_list).block(workspace_block);
        Widget::render(list_widget, area, buf);
    }
}
