use std::{cell::RefCell, rc::Rc};

use chrono::{DateTime, Local};
use ratatui::{
    style::{Color, Modifier, Style, Styled, Stylize},
    text::Line,
    widgets::{Block, List, ListItem, ListState, Padding, StatefulWidget, Widget},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::ui::{SelectAction, SelectBF, workspacewidget::Workspace};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProcess,
    Finished,
    Deprecated,
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
    pub fn set_task_status(task: &Rc<RefCell<Task>>, status: TaskStatus) {
        let mut task_mut = task.borrow_mut();
        task_mut.status = status.clone();
        if !task_mut.children.is_empty() {
            task_mut.children.iter().for_each(|t| {
                Task::set_task_status(t, status.clone());
            });
        }
    }

    pub fn rename(&mut self, new_name: String) {
        self.desc = new_name;
    }

    // TODO: use regex to completed the search functionality
    pub fn is_target(&self, search_string: String) -> bool {
        let search_strings = search_string.split(" ");
        let mut result = false;
        search_strings.into_iter().for_each(|s| {
            if self.desc.contains(s) {
                result = true;
            }
        });
        for task in self.children.iter() {
            if task.borrow().is_target(search_string.to_owned()) {
                result = true;
                break;
            }
        }
        result
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

    pub fn refresh_current_task(&mut self) {
        let mut res = None;
        if let Some(cur_task) = &self.current_task {
            let tasks = TodoList::get_flattened(&self.tasks);
            for task in tasks.iter() {
                if cur_task.borrow().id == task.borrow().id {
                    res = Some(task.clone());
                    break;
                }
            }
            self.current_task = res;
        }
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
                    TodoList::delete_item(cur_task, &mut task_mut.children);
                }
            }
        }
        if let Some(i) = res {
            tasks.remove(i);
        }
    }

    pub fn delete_task(&mut self) {
        if let Some(cur_task) = &self.current_task {
            TodoList::delete_item(cur_task, &mut self.tasks);
        }
        self.current_task = None;
        self.state.select(None);
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
    pub fn get_task_list_item<'a>(
        task_list: &[Rc<RefCell<Task>>],
        dep: usize,
    ) -> Vec<ListItem<'a>> {
        let mut task_item = Vec::<ListItem>::new();
        task_list.iter().for_each(|item| {
            let task = item.borrow();
            let desc = task.desc.to_owned();
            let prefix = match &task.status {
                TaskStatus::Todo => "▢".white(),
                TaskStatus::InProcess => "▣".blue(),
                TaskStatus::Finished => "✓".green(),
                TaskStatus::Deprecated => "".red(),
            };
            let it = ListItem::new(Line::from(vec![
                prefix,
                "  ".repeat(dep).into(),
                //     .set_style(match &task.status {
                //     TaskStatus::Finished => Style::new()
                //         .add_modifier(Modifier::CROSSED_OUT)
                //         .fg(Color::LightGreen),
                //     TaskStatus::Deprecated => Style::new()
                //         .add_modifier(Modifier::CROSSED_OUT)
                //         .fg(Color::Red),
                //     _ => Style::default(),
                // }),
                desc.set_style(match &task.status {
                    TaskStatus::Finished => Style::new()
                        // .add_modifier(Modifier::CROSSED_OUT)
                        .fg(Color::LightGreen),
                    TaskStatus::Deprecated => Style::new()
                        .add_modifier(Modifier::CROSSED_OUT)
                        .fg(Color::Red),
                    _ => Style::default(),
                }),
            ]));
            task_item.push(it);

            if task.expanded {
                let child = TodoWidget::get_task_list_item(&task.children, dep + 1);
                task_item.extend(child);
            }
        });

        task_item
    }

    pub fn get_search_list_item<'a>(
        search_string: String,
        task_list: &[Rc<RefCell<Task>>],
        dep: usize,
    ) -> Vec<ListItem<'a>> {
        let mut task_item = Vec::<ListItem>::new();
        task_list.iter().for_each(|item| {
            let task = item.borrow();
            let desc = task.desc.to_owned();
            let prefix = match &task.status {
                TaskStatus::Todo => "▢".white(),
                TaskStatus::InProcess => "▣".blue(),
                TaskStatus::Finished => "✓".green(),
                TaskStatus::Deprecated => "".red(),
            };
            let mut contents = vec![prefix, "  ".repeat(dep).into()];
            if !search_string.is_empty() {
                let search_strings = search_string.split(" ");
                let mut idx_str: Vec<(usize, &str)> = Vec::new();
                search_strings.for_each(|s| {
                    let mut v: Vec<(usize, &str)> = desc.match_indices(s).collect();
                    idx_str.append(&mut v);
                });
                idx_str.sort_by(|a, b| a.0.cmp(&b.0));
                let mut idx_str_merged: Vec<(usize, usize)> = Vec::new();
                for (idx, s) in idx_str {
                    if let Some(last) = idx_str_merged.last_mut()
                        && idx < last.1
                    {
                        last.1 = last.1.max(idx + s.len());
                        continue;
                    }
                    idx_str_merged.push((idx, idx + s.len()));
                }
                let mut cursor = 0;
                for (s, e) in idx_str_merged {
                    if s > cursor {
                        contents.push(
                            desc[cursor..s].to_owned().set_style(match &task.status {
                                TaskStatus::Finished => Style::new().fg(Color::LightGreen),
                                TaskStatus::Deprecated => Style::new()
                                    .add_modifier(Modifier::CROSSED_OUT)
                                    .fg(Color::Red),
                                _ => Style::default(),
                            }),
                        );
                    }
                    contents.push(
                        desc[s..e]
                            .to_owned()
                            .light_yellow()
                            .add_modifier(Modifier::ITALIC),
                    );
                    cursor = e;
                }
                if cursor < desc.len() {
                    contents.push(
                        desc[cursor..].to_owned().set_style(match &task.status {
                            TaskStatus::Finished => Style::new()
                                // .add_modifier(Modifier::CROSSED_OUT)
                                .fg(Color::LightGreen),
                            TaskStatus::Deprecated => Style::new()
                                .add_modifier(Modifier::CROSSED_OUT)
                                .fg(Color::Red),
                            _ => Style::default(),
                        }),
                    );
                }
            } else {
                contents.push(
                    desc.set_style(match &task.status {
                        TaskStatus::Finished => Style::new().fg(Color::LightGreen),
                        TaskStatus::Deprecated => Style::new()
                            .add_modifier(Modifier::CROSSED_OUT)
                            .fg(Color::Red),
                        _ => Style::default(),
                    }),
                );
            }
            // let mut spans = Vec::new();
            // spans.push(prefix);
            // spans.push("  ".repeat(dep).into());
            // search_strings.clone().for_each(|s| {
            //     if !s.is_empty() {
            //         let mut tar_idx = 0;
            //         for (idx, tar_str) in desc.match_indices(s) {
            //             if idx > tar_idx {
            //                 spans.push(Span::raw(&desc[tar_idx..idx]));
            //             }
            //             tar_idx = idx + tar_str.len();
            //             spans.push(Span::styled(
            //                 tar_str,
            //                 Style::new()
            //                     .fg(Color::LightYellow)
            //                     .add_modifier(Modifier::ITALIC),
            //             ));
            //         }
            //         if tar_idx < s.len() {
            //             spans.push(Span::raw(&desc[tar_idx..]));
            //         }
            //     }
            // });

            let it = ListItem::new(Line::from(contents));
            // let it = ListItem::new(Line::from(vec![
            //     prefix,
            //     "  ".repeat(dep).into(),
            //     //     .set_style(match &task.status {
            //     //     TaskStatus::Finished => Style::new()
            //     //         .add_modifier(Modifier::CROSSED_OUT)
            //     //         .fg(Color::LightGreen),
            //     //     TaskStatus::Deprecated => Style::new()
            //     //         .add_modifier(Modifier::CROSSED_OUT)
            //     //         .fg(Color::Red),
            //     //     _ => Style::default(),
            //     // }),
            //     desc.set_style(match &task.status {
            //         TaskStatus::Finished => Style::new()
            //             // .add_modifier(Modifier::CROSSED_OUT)
            //             .fg(Color::LightGreen),
            //         TaskStatus::Deprecated => Style::new()
            //             .add_modifier(Modifier::CROSSED_OUT)
            //             .fg(Color::Red),
            //         _ => Style::default(),
            //     }),
            // ]));
            task_item.push(it);

            if task.expanded {
                let child = TodoWidget::get_task_list_item(&task.children, dep + 1);
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
            target.borrow_mut().refresh_current_task();
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
            .title(" <3> Todo List ".blue())
            .border_style(if self.focused {
                Style::new().fg(Color::Blue)
            } else {
                Style::new().fg(Color::DarkGray)
            })
            .padding(Padding::uniform(1));

        let todo_listitems = Vec::<ListItem>::new();
        if let Some(todolist) = &self.current_todolist {
            let tasks = todolist.borrow().tasks.to_owned();
            let task_list = TodoWidget::get_task_list_item(&tasks, 1);
            // task_list.iter().for_each(|task| {
            //     todo_listitems.push(ListItem::new(task));
            // });
            let listwidget = List::new(task_list)
                .block(block)
                .highlight_style(if self.focused {
                    Style::new().bg(Color::Rgb(66, 80, 102))
                } else {
                    Style::new()
                });
            // let state = &mut todolist.borrow_mut().state;
            let state = &mut todolist.borrow_mut().state;

            StatefulWidget::render(listwidget, area, buf, state);
        } else {
            let listwidget =
                List::new(todo_listitems)
                    .block(block)
                    .highlight_style(if self.focused {
                        Style::new().bg(Color::Rgb(80, 100, 109))
                    } else {
                        Style::new()
                    });
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
