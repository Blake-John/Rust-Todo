use std::{cell::RefCell, rc::Rc};

use chrono::{DateTime, Local};
use ratatui::{
    style::{Color, Style, Stylize},
    widgets::{Block, List, ListItem, ListState, StatefulWidget, Widget},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::ui::{SelectAction, SelectBF, workspacewidget::Workspace};

#[derive(Debug, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProcess,
    Finished,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub desc: String,
    pub status: TaskStatus,
    pub expanded: bool,
    pub due: Option<DateTime<Local>>,
    pub children: Vec<Rc<RefCell<Task>>>,
    pub id: Uuid,
}

impl Task {
    pub fn new(desc: String, due: Option<DateTime<Local>>) -> Self {
        Self {
            desc,
            status: TaskStatus::Todo,
            expanded: true,
            due,
            children: Vec::new(),
            id: Uuid::new_v4(),
        }
    }

    pub fn add_child(&mut self, task: Rc<RefCell<Task>>) {
        self.children.push(task);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoList {
    pub workspace: Uuid,
    pub tasks: Vec<Rc<RefCell<Task>>>,
    pub current_task: Option<Rc<RefCell<Task>>>,
    #[serde(skip_serializing, skip_deserializing)]
    pub state: ListState,
}

impl TodoList {
    pub fn new(ws_id: Uuid) -> Self {
        Self {
            workspace: ws_id,
            tasks: Vec::new(),
            current_task: None,
            state: ListState::default(),
        }
    }

    pub fn add_task(&mut self, task: Rc<RefCell<Task>>) {
        self.tasks.push(task.clone());
        self.current_task = Some(task);
    }

    pub fn add_child_task(&mut self, task: Rc<RefCell<Task>>) {
        if let Some(ctask) = &self.current_task {
            ctask.borrow_mut().add_child(task);
        } else {
            self.add_task(task.clone());
            self.current_task = Some(task);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TodoWidget {
    pub todolists: Vec<Rc<RefCell<TodoList>>>,
    pub current_todolist: Option<Rc<RefCell<TodoList>>>,
    pub focused: bool,
}

impl TodoWidget {
    pub fn new() -> Self {
        Self {
            todolists: Vec::new(),
            current_todolist: None,
            focused: false,
        }
    }
    pub fn get_task_list(task_list: &Vec<Rc<RefCell<Task>>>, dep: usize) -> Vec<(String, String)> {
        let mut task_item = Vec::<(String, String)>::new();
        task_list.iter().for_each(|item| {
            let task = item.borrow();
            let desc = task.desc.to_owned();
            let id = task.id.to_string();
            let it = "  ".repeat(dep) + desc.as_str();
            task_item.push((it, id));

            if task.expanded {
                let child = TodoWidget::get_task_list(&task.children, dep + 1);
                task_item.extend(child);
            }
        });

        task_item
    }

    pub fn change_current_list(&mut self, workspace: &Option<Rc<RefCell<Workspace>>>) {
        if let Some(cws) = workspace {
            let ws_id = cws.borrow().id.to_string();
            let target = self
                .todolists
                .iter()
                .find(|&l| l.borrow().workspace.to_string() == ws_id)
                .unwrap()
                .to_owned();
            self.current_todolist = Some(target);
        }
    }

    pub fn add_list(&mut self, list: Rc<RefCell<TodoList>>) {
        self.todolists.push(list);
    }
}

impl Widget for &mut TodoWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered()
            .title(" Todo List ".blue())
            .border_style(if self.focused {
                Style::new().fg(Color::Blue)
            } else {
                Style::default()
            });

        let mut todo_listitems = Vec::<ListItem>::new();
        if let Some(todolist) = &self.current_todolist {
            // let ct_id = if let Some(ct) = &todolist.borrow().current_task {
            //     ct.borrow().id.to_owned().to_string()
            // } else {
            //     "".to_string()
            // };
            let tasks = todolist.borrow().tasks.to_owned();
            let task_list = TodoWidget::get_task_list(&tasks, 0);
            task_list.iter().for_each(|(task, _)| {
                todo_listitems.push(ListItem::new(task.to_owned()));
            });
            let listwidget = List::new(todo_listitems)
                .block(block)
                .highlight_style(Style::new().bg(Color::Blue).fg(Color::Black));
            let state = &mut todolist.borrow_mut().state;

            StatefulWidget::render(listwidget, area, buf, state);
        } else {
            let listwidget = List::new(todo_listitems)
                .block(block)
                .highlight_style(Style::new().bg(Color::Blue).fg(Color::Black));
            Widget::render(listwidget, area, buf);
        }
    }
}

impl SelectAction<Task> for TodoList {
    fn get_selected_bf(
        current_target: &Option<Rc<RefCell<Task>>>,
        targets: &Vec<Rc<RefCell<Task>>>,
        state: &mut ListState,
        bf: super::SelectBF,
    ) -> Option<Rc<RefCell<Task>>> {
        let task_list = TodoList::get_flattened(targets);
        if task_list.len() > 0 {
            if current_target.is_none() {
                state.select(Some(0));
                Some(task_list[0].clone())
            } else {
                let mut target = 0;

                if let Some(cw) = current_target {
                    let (i, _) = task_list
                        .iter()
                        .enumerate()
                        .find(|(_, ws)| ws.borrow().desc == cw.borrow().desc)
                        .unwrap();
                    target = i;
                }
                match bf {
                    SelectBF::Back => {
                        state.select_previous();
                        if target != 0 {
                            target -= 1;
                        }
                    }
                    SelectBF::Forward => {
                        state.select_next();
                        if target < task_list.len() - 1 {
                            target += 1;
                        }
                    }
                }

                Some(task_list[target].clone())
            }
        } else {
            None
        }
    }

    fn get_flattened(target: &Vec<Rc<RefCell<Task>>>) -> Vec<Rc<RefCell<Task>>> {
        let mut result = Vec::<Rc<RefCell<Task>>>::new();
        target.iter().for_each(|ws| {
            result.push(ws.clone());
            let ws_ = ws.borrow();
            if !ws_.children.is_empty() {
                let child = TodoList::get_flattened(&ws_.children);
                result.extend(child);
            }
        });

        result
    }
}
