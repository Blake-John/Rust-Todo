use std::{cell::RefCell, rc::Rc};

use ratatui::{
    style::{Color, Style, Stylize},
    widgets::{Block, List, ListItem, ListState, Widget},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub desc: String,
    pub expanded: bool,
    pub children: Vec<Rc<RefCell<Workspace>>>,
}

impl Workspace {
    pub fn new(desc: String) -> Self {
        Self {
            desc: desc,
            expanded: true,
            children: Vec::<Rc<RefCell<Workspace>>>::new(),
        }
    }
    pub fn add_child(&mut self, workspace: Rc<RefCell<Self>>) {
        self.children.push(workspace);
    }

    pub fn add_children(&mut self, workspaces: Vec<Rc<RefCell<Self>>>) {
        workspaces.iter().for_each(|workspace| {
            self.add_child(workspace.clone());
        });
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceWidget {
    pub workspaces: Vec<Rc<RefCell<Workspace>>>,
    pub current_workspace: Option<Rc<RefCell<Workspace>>>,
    pub focused: bool,
}

impl WorkspaceWidget {
    pub fn new() -> Self {
        Self {
            workspaces: Vec::<Rc<RefCell<Workspace>>>::new(),
            current_workspace: None,
            focused: true,
        }
    }

    pub fn add_workspace(&mut self, workspace: Rc<RefCell<Workspace>>) {
        self.workspaces.push(workspace.clone());
        self.current_workspace = Some(workspace);
    }

    pub fn add_child_workspace(&mut self, workspace: Rc<RefCell<Workspace>>) {
        if let Some(current_workspace) = &self.current_workspace {
            let mut cw = current_workspace.borrow_mut();
            cw.add_child(workspace.clone());
        } else {
            self.add_workspace(workspace.clone());
        }
        self.current_workspace = Some(workspace.clone());
    }

    pub fn get_ws_list(workspaces: Vec<Rc<RefCell<Workspace>>>, dep: usize) -> Vec<String> {
        let mut list_item = Vec::<String>::new();
        for item in workspaces.iter() {
            let ws = item.borrow().to_owned();
            let desc = ws.desc.clone();
            let it = "  ".repeat(dep) + desc.as_str();
            list_item.push(it);

            if ws.expanded {
                let children_list = WorkspaceWidget::get_ws_list(ws.children.clone(), dep + 1);
                list_item.extend(children_list);
            }
        }

        list_item
    }
}

impl Widget for &mut WorkspaceWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let ws_list = WorkspaceWidget::get_ws_list(self.workspaces.clone(), 0);
        let mut workspace_list = Vec::<ListItem>::new();
        let cws_desc = if let Some(cw) = &self.current_workspace {
            cw.borrow().desc.to_owned()
        } else {
            "".to_string()
        };
        ws_list.iter().for_each(|desc| {
            if desc.trim() == cws_desc {
                workspace_list
                    .push(ListItem::new(desc.to_owned().fg(Color::Black)).bg(Color::Green));
            } else {
                workspace_list.push(ListItem::new(desc.to_owned()));
            }
        });

        let workspace_block =
            Block::bordered()
                .title(" Workspace ".green())
                .border_style(if self.focused {
                    Style::new().fg(Color::LightGreen)
                } else {
                    Style::default()
                });

        let list_widget = List::new(workspace_list).block(workspace_block);
        Widget::render(list_widget, area, buf);
    }
}
