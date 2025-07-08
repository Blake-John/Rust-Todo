use std::{cell::RefCell, rc::Rc};

use chrono::{DateTime, Local};
use ratatui::{
    style::{Color, Style, Stylize},
    widgets::{Block, List, ListItem, ListState, Padding, StatefulWidget, Widget},
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
    #[serde(default)]
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
    }

    pub fn add_child_task(&mut self, task: Rc<RefCell<Task>>) {
        if let Some(ctask) = &self.current_task {
            ctask.borrow_mut().add_child(task);
        } else {
            self.add_task(task.clone());
            // self.current_task = Some(task);
        }
    }

    pub fn set_current_task_none(&mut self) {
        self.state.select(None);
        self.current_task = None;
    }

    pub fn delete_item(cur_task: &Rc<RefCell<Task>>, tasks: &mut Vec<Rc<RefCell<Task>>>) {
        let mut res = None;
        for (i, task) in tasks.iter().enumerate() {
            if Rc::ptr_eq(cur_task, task) {
                res = Some(i);
                break;
            } else {
                let mut task_mut = task.borrow_mut();
                if !task_mut.children.is_empty() {
                    print!("find in sub");
                    TodoList::delete_item(cur_task, &mut task_mut.children);
                }
            }
        }
        print!("{:?}", res);
        if let Some(i) = res {
            tasks.remove(i);
        }
        // let mut res = None;
        // for (i, task) in tasks.iter().enumerate() {
        //     let mut task_mut = task.borrow_mut();
        //     if task_mut.id == cur_task.borrow().id {
        //         res = Some(i);
        //         break;
        //     }
        //     if !task_mut.children.is_empty() {
        //         TodoList::delete_item(cur_task, &mut task_mut.children);
        //     }
        // }
        // // println!("{:?}", res);
        // if let Some(i) = res {
        //     tasks.remove(i);
        // }
    }

    pub fn delete_task(&mut self) {
        if let Some(cur_task) = &self.current_task {
            TodoList::delete_item(cur_task, &mut self.tasks);
        }
        self.current_task = None;
        self.state.select(None);
        // let list_id = cur_list.borrow().workspace;
        // for list in lists.iter() {
        //     let tar_list_id = list.clone().borrow().workspace;
        //     if tar_list_id == list_id {
        //         let mut list_mut = list.borrow_mut();
        //         // let cur_task_ = cur_task.clone();
        //         TodoList::delete_item(cur_task, &mut list_mut.tasks);
        //     }
        // }
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

    pub fn set_cur_task_none(&mut self) {
        if let Some(cur_list) = &self.current_todolist {
            let mut cur_list_mut = cur_list.borrow_mut();
            cur_list_mut.set_current_task_none();
        }
    }

    pub fn add_list(&mut self, list: Rc<RefCell<TodoList>>) {
        self.todolists.push(list);
    }

    pub fn delete_list(&mut self, tar_ws: Uuid) {
        let res = self
            .todolists
            .iter()
            .enumerate()
            .find(|(_, list)| list.borrow().workspace == tar_ws);
        if let Some((i, _)) = res {
            self.todolists.remove(i);
        }
        self.current_todolist = None;
    }
}

impl Default for TodoWidget {
    fn default() -> Self {
        Self::new()
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
                Style::new().fg(Color::DarkGray)
            })
            .padding(Padding::uniform(1));

        let mut todo_listitems = Vec::<ListItem>::new();
        if let Some(todolist) = &self.current_todolist {
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
        if !task_list.is_empty() {
            if current_target.is_none() {
                state.select(Some(0));
                Some(task_list[0].clone())
            } else {
                let mut target = 0;

                if let Some(ct) = current_target {
                    let (i, _) = task_list
                        .iter()
                        .enumerate()
                        .find(|(_, task)| task.borrow().id == ct.borrow().id)
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
