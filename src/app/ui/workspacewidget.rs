use std::{cell::RefCell, rc::Rc};

use ratatui::{
    style::{Color, Style, Stylize},
    widgets::{Block, List, ListItem, ListState, Padding, StatefulWidget, Widget},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::ui::{SelectAction, SelectBF};

/// The Workspace Structure to store the basic information of a workspace
///
/// # Fields
///
/// - `desc` (`String`) - the description of the workspace.
/// - `id` (`Uuid`) - unique id of the workspace.
/// - `expanded` (`bool`) - whether the workspace is expanded or not.
/// - `children` (`Vec<Rc<RefCell<Workspace>>>`) - the children/sub ws of the workspace, is a vector of workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub desc: String,
    pub id: Uuid,
    pub expanded: bool,
    pub children: Vec<Rc<RefCell<Workspace>>>,
}

impl Workspace {
    pub fn new(desc: String) -> Self {
        Self {
            desc,
            id: Uuid::new_v4(),
            expanded: true,
            children: Vec::<Rc<RefCell<Workspace>>>::new(),
        }
    }
    /// add the child to the [`Workspace::children`] component of [`Workspace`]
    ///
    /// # Arguments
    ///
    /// - `&mut self` ([`Workspace`])
    /// - `workspace` (`Rc<RefCell<Self>>`) - workspace to add as a child.
    pub fn add_child(&mut self, workspace: Rc<RefCell<Self>>) {
        self.children.push(workspace);
    }

    /// add multiple children to the [`Workspace::children`] component of [`Workspace`]
    ///
    /// # Arguments
    ///
    /// - `&mut self` ([`Workspace`])
    /// - `workspaces` (`Vec<Rc<RefCell<Self>>>`) - workspaces to add as children.
    pub fn add_children(&mut self, workspaces: Vec<Rc<RefCell<Self>>>) {
        workspaces.iter().for_each(|workspace| {
            self.add_child(workspace.clone());
        });
    }
}

/// The Widget to display workspaces.
///
/// # Fields
///
/// - `workspaces` (`Vec<Rc<RefCell<Workspace>>>`) - a vector of [`Workspace`]
/// - `current_workspace` (`Option<Rc<RefCell<Workspace>>>`) - the current selected workspace or None.
/// - `focused` (`bool`) - whether the widget is focused or not.
/// - `#[serde(default)] ws_state` (`ListState`) - The [`ListState`] of the [`List`] widget, which is used to select the workspace
/// because the workspaces are displayed in a [`List`] widget.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceWidget {
    pub workspaces: Vec<Rc<RefCell<Workspace>>>,
    pub current_workspace: Option<Rc<RefCell<Workspace>>>,
    pub focused: bool,
    #[serde(default)]
    pub ws_state: ListState,
}

impl WorkspaceWidget {
    pub fn new() -> Self {
        Self {
            workspaces: Vec::<Rc<RefCell<Workspace>>>::new(),
            current_workspace: None,
            focused: true,
            ws_state: ListState::default(),
        }
    }

    /// add a workspace to the widget
    ///
    /// # Arguments
    ///
    /// - `&mut self` ([`WorkspaceWidget`])
    /// - `workspace` (`Rc<RefCell<Workspace>>`) - a workspace to add
    pub fn add_workspace(&mut self, workspace: Rc<RefCell<Workspace>>) {
        self.workspaces.push(workspace.clone());
    }

    /// add a child workspace to the widget based on the [`WorkspaceWidget::current_workspace`] field
    ///
    /// # Arguments
    ///
    /// - `&mut self` ([`WorkspaceWidget`])
    /// - `workspace` (`Rc<RefCell<Workspace>>`) - the target workspace to add as a child
    pub fn add_child_workspace(&mut self, workspace: Rc<RefCell<Workspace>>) {
        if let Some(current_workspace) = &self.current_workspace {
            let mut cw = current_workspace.borrow_mut();
            cw.add_child(workspace.clone());
        } else {
            self.add_workspace(workspace.clone());
        }
        // self.current_workspace = Some(workspace.clone());
    }

    /// Refresh the [`WorkspaceWidget::current_workspace`] to make it consistent with the
    /// [`WorkspaceWidget::workspaces`] in memory while first load the data from file
    ///
    /// This is because that I use the `.json` file to store the data, and [`Rc<RefCell<Workspace>`] to store the data in memory.
    /// When first time load the data from file, `current_workspace` will be a completely different object from any one of the
    /// `workspaces` in memory, although they have the same value.
    pub fn refresh_current(&mut self) {
        if let Some(cur_ws) = self.current_workspace.clone() {
            let new_ws_list = Workspace::get_flattened(&self.workspaces);
            for ws in new_ws_list.iter() {
                let cur_ws_id = cur_ws.borrow_mut().id;
                if cur_ws_id == ws.borrow().id {
                    self.current_workspace = Some(ws.clone());
                }
            }
        }
    }

    /// a function to get the workspaces' desc list in the form of a list of strings,
    /// whose return value is used to construct a [`List`] widget to display the workspaces
    ///
    /// # Arguments
    ///
    /// - `workspaces` (`&Vec<Rc<RefCell<Workspace>>>`) - the workspaces to get the desc list from
    /// - `dep` (`usize`) - a helper parameter to determine the depth of the workspaces, because each worksapce might
    /// have children and this function will be called recursively
    ///
    /// # Returns
    ///
    /// - `Vec<String>` - the desc list of the workspaces, which is indented
    pub fn get_ws_list(workspaces: &Vec<Rc<RefCell<Workspace>>>, dep: usize) -> Vec<String> {
        let mut list_item = Vec::<String>::new();
        workspaces.iter().for_each(|item| {
            let ws = item.borrow();
            let desc = ws.desc.clone();
            let it = "  ".repeat(dep) + desc.as_str();
            list_item.push(it);

            if ws.expanded {
                let children_list = WorkspaceWidget::get_ws_list(&ws.children, dep + 1);
                list_item.extend(children_list);
            }
        });

        list_item
    }

    /// Delete a workspace from the [`WorkspaceWidget::workspaces`] field
    ///
    /// # Arguments
    ///
    /// - `workspaces` (`&mut Vec<Rc<RefCell<Workspace>>>`) - to be deleted
    /// - `cur_ws` (`&Option<Rc<RefCell<Workspace>>>`) - what to delete from
    pub fn delete_item(
        workspaces: &mut Vec<Rc<RefCell<Workspace>>>,
        cur_ws: &Rc<RefCell<Workspace>>,
    ) {
        let mut result = None;
        for (i, ws) in workspaces.iter().enumerate() {
            if Rc::ptr_eq(ws, cur_ws) {
                result = Some(i);
                break;
            } else {
                let mut ws_mut = ws.borrow_mut();
                if !ws_mut.children.is_empty() {
                    WorkspaceWidget::delete_item(&mut ws_mut.children, cur_ws);
                }
            }
        }

        if let Some(i) = result {
            workspaces.remove(i);
        }
    }
}

impl Default for WorkspaceWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for &mut WorkspaceWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let ws_list = WorkspaceWidget::get_ws_list(&self.workspaces, 0);
        let mut workspace_list = Vec::<ListItem>::new();
        ws_list.iter().for_each(|desc| {
            workspace_list.push(ListItem::new(desc.to_owned()));
        });

        let workspace_block = Block::bordered()
            .title(" Workspace ".green())
            .border_style(if self.focused {
                Style::new().fg(Color::LightGreen)
            } else {
                Style::new().fg(Color::DarkGray)
            })
            .padding(Padding::uniform(1));

        let list_widget = List::new(workspace_list)
            .block(workspace_block)
            .highlight_style(Style::new().fg(Color::Black).bg(Color::Green));
        StatefulWidget::render(list_widget, area, buf, &mut self.ws_state);
    }
}

impl SelectAction<Self> for Workspace {
    fn get_selected_bf(
        current_target: &Option<Rc<RefCell<Workspace>>>,
        targets: &Vec<Rc<RefCell<Workspace>>>,
        state: &mut ListState,
        bf: super::SelectBF,
    ) -> Option<Rc<RefCell<Workspace>>> {
        let ws_list = Workspace::get_flattened(targets);
        if !ws_list.is_empty() {
            if current_target.is_none() {
                state.select(Some(0));
                Some(ws_list[0].clone())
            } else {
                let mut target = 0;

                if let Some(cw) = current_target {
                    let (i, _) = ws_list
                        .iter()
                        .enumerate()
                        .find(|(_, ws)| ws.borrow().id == cw.borrow().id)
                        .unwrap();
                    target = i;
                }
                match bf {
                    SelectBF::Back => {
                        state.select_previous();
                        target = target.saturating_sub(1);
                    }
                    SelectBF::Forward => {
                        state.select_next();
                        if target < ws_list.len() - 1 {
                            target += 1;
                        }
                    }
                }

                Some(ws_list[target].clone())
            }
        } else {
            state.select(None);
            None
        }
    }

    fn get_flattened(target: &Vec<Rc<RefCell<Workspace>>>) -> Vec<Rc<RefCell<Workspace>>> {
        let mut result = Vec::<Rc<RefCell<Workspace>>>::new();
        target.iter().for_each(|ws| {
            result.push(ws.clone());
            let ws_ = ws.borrow();
            if !ws_.children.is_empty() {
                let child = Workspace::get_flattened(&ws_.children);
                result.extend(child);
            }
        });

        result
    }
}
