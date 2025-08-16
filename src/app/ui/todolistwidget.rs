use chrono::{Local, NaiveDate};
use ratatui::{
    style::{Color, Modifier, Style, Styled, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, Padding, StatefulWidget, Widget},
};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, rc::Rc};
use uuid::Uuid;

use crate::app::ui::{SelectAction, SelectBF, workspacewidget::Workspace};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProcess,
    Finished,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Urgency {
    Critical,
    Important,
    Common,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub desc: String,
    pub status: TaskStatus,
    pub expanded: bool,
    pub due: Option<NaiveDate>,
    pub children: Vec<Rc<RefCell<Task>>>,
    pub id: Uuid,
    pub urgency: Option<Urgency>,
}

impl Task {
    pub fn new(desc: String, due: Option<NaiveDate>) -> Self {
        Self {
            desc,
            status: TaskStatus::Todo,
            expanded: true,
            due,
            children: Vec::new(),
            id: Uuid::new_v4(),
            urgency: None,
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
            let tasks = TodoWidget::get_flattened(&self.tasks);
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodoWidget {
    pub todolists: Vec<Rc<RefCell<TodoList>>>,
    pub current_todolist: Option<Rc<RefCell<TodoList>>>,
    pub focused: bool,

    #[serde(skip)]
    #[serde(default)]
    pub search_string: String,
}

impl TodoWidget {
    pub fn new() -> Self {
        Self {
            todolists: Vec::new(),
            current_todolist: None,
            focused: false,
            search_string: String::new(),
        }
    }

    pub fn find_max_tasks_len(task_list: &[Rc<RefCell<Task>>], dep: usize) -> usize {
        let mut max_len = 0;
        task_list.iter().for_each(|item| {
            max_len = max_len.max(item.borrow().desc.len() + dep * 2_usize);
            if !item.borrow().children.is_empty() {
                max_len = max_len.max(TodoWidget::find_max_tasks_len(
                    &item.borrow().children,
                    dep + 1,
                ));
            }
        });

        max_len
    }

    pub fn get_task_list_item<'a>(
        task_list: &[Rc<RefCell<Task>>],
        dep: usize,
        max_desc_len: usize,
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
            let mut due_span = Span::raw("");
            if let Some(due) = item.borrow().due {
                let delta = due - Local::now().date_naive();
                let num_days = delta.num_days();
                match &task.status {
                    TaskStatus::Todo | TaskStatus::InProcess => {
                        due_span = match num_days {
                            ..0 => format!(" {} day over ! ", num_days.abs())
                                .to_string()
                                .set_style(Style::new().fg(Color::Yellow)),
                            0 => format!(" {} day left ! ", num_days)
                                .to_string()
                                .set_style(Style::new().fg(Color::Red)),
                            1 => format!(" {} day left ! ", num_days)
                                .to_string()
                                .set_style(Style::new().fg(Color::LightRed)),
                            2..4 => format!(" {} day left ! ", num_days)
                                .to_string()
                                .set_style(Style::new().fg(Color::Yellow)),
                            4..7 => format!(" {} day left ! ", num_days)
                                .to_string()
                                .set_style(Style::new().fg(Color::LightBlue)),
                            7.. => format!(" {} day left ! ", num_days)
                                .to_string()
                                .set_style(Style::new().fg(Color::LightGreen)),
                        };
                    }
                    _ => {}
                }
            }
            let padding_len = max_desc_len - desc.len() - dep * 2 + 1;
            let it = ListItem::new(Line::from(vec![
                prefix,
                "  ".repeat(dep).into(),
                //     .set_style(match &task.status {
                //     // TaskStatus::Finished => Style::new()
                //     //     .add_modifier(Modifier::CROSSED_OUT)
                //     //     .fg(Color::LightGreen),
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
                format!("{:padding_len$}", " ").into(),
                "    ".into(),
                due_span,
            ]));
            task_item.push(it);

            if task.expanded {
                let child = TodoWidget::get_task_list_item(&task.children, dep + 1, max_desc_len);
                task_item.extend(child);
            }
        });

        task_item
    }

    pub fn get_search_list_item<'a>(
        search_string: String,
        task_list: &[Rc<RefCell<Task>>],
        dep: usize,
        max_desc_len: usize,
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

            let mut due_span = Span::raw("");
            if let Some(due) = item.borrow().due {
                let delta = due - Local::now().date_naive();
                let num_days = delta.num_days();
                match &task.status {
                    TaskStatus::Todo | TaskStatus::InProcess => {
                        due_span = match num_days {
                            ..0 => format!(" {} day over ! ", num_days.abs())
                                .to_string()
                                .set_style(Style::new().fg(Color::Yellow)),
                            0 => format!(" {} day left ! ", num_days)
                                .to_string()
                                .set_style(Style::new().fg(Color::Red)),
                            1 => format!(" {} day left ! ", num_days)
                                .to_string()
                                .set_style(Style::new().fg(Color::LightRed)),
                            2..4 => format!(" {} day left ! ", num_days)
                                .to_string()
                                .set_style(Style::new().fg(Color::Yellow)),
                            4..7 => format!(" {} day left ! ", num_days)
                                .to_string()
                                .set_style(Style::new().fg(Color::LightBlue)),
                            7.. => format!(" {} day left ! ", num_days)
                                .to_string()
                                .set_style(Style::new().fg(Color::LightGreen)),
                        };
                    }
                    _ => {}
                }
            }
            let padding_len = max_desc_len - desc.len() - dep * 2 + 1;

            if !search_string.is_empty() {
                let search_strings = search_string.split(" ");
                let mut idx_str: Vec<(usize, &str)> = Vec::new();
                search_strings.for_each(|s| {
                    let mut v: Vec<(usize, &str)> = desc.match_indices(s).collect();
                    idx_str.append(&mut v);
                });
                if !idx_str.is_empty() {
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
            contents.extend(vec![
                format!("{:padding_len$}", " ").into(),
                "    ".into(),
                due_span,
            ]);

            let it = ListItem::new(Line::from(contents.clone()));
            task_item.push(it);
            let child = TodoWidget::get_search_list_item(
                search_string.to_owned(),
                &task.children,
                dep + 1,
                max_desc_len,
            );
            task_item.extend(child);

            // if !contents.is_empty() {
            //     let it = ListItem::new(Line::from(contents.clone()));
            //     task_item.push(it);
            //     if !task.children.is_empty() {
            //         let child = TodoWidget::get_task_list_item(&task.children, dep + 1);
            //         task_item.extend(child);
            //     }
            // } else if !task.children.is_empty() {
            //     let child = TodoWidget::get_search_list_item(
            //         search_string.to_owned(),
            //         &task.children,
            //         dep + 1,
            //     );
            //     if !child.is_empty() {
            //         let it = ListItem::new(Line::from(contents.clone()));
            //         task_item.push(it);
            //     }
            //     task_item.extend(child);
            // }
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
            if self.search_string.is_empty() {
                let tasks = todolist.borrow().tasks.to_owned();
                let max_desc_len = TodoWidget::find_max_tasks_len(&tasks, 1);
                let task_list = TodoWidget::get_task_list_item(&tasks, 1, max_desc_len);
                let listwidget =
                    List::new(task_list)
                        .block(block)
                        .highlight_style(if self.focused {
                            Style::new().bg(Color::Rgb(66, 80, 102))
                        } else {
                            Style::new()
                        });
                let state = &mut todolist.borrow_mut().state;

                StatefulWidget::render(listwidget, area, buf, state);
            } else {
                let mut tar_list = Vec::new();

                todolist.borrow().tasks.iter().for_each(|task| {
                    if task.borrow().is_target(self.search_string.clone()) {
                        tar_list.push(task.to_owned());
                    }
                });
                let max_desc_len = TodoWidget::find_max_tasks_len(&tar_list, 1);
                let task_list = TodoWidget::get_search_list_item(
                    self.search_string.clone(),
                    &tar_list,
                    1,
                    max_desc_len,
                );
                let listwidget =
                    List::new(task_list)
                        .block(block)
                        .highlight_style(if self.focused {
                            Style::new().bg(Color::Rgb(66, 80, 102))
                        } else {
                            Style::new()
                        });
                let state = &mut todolist.borrow_mut().state;

                StatefulWidget::render(listwidget, area, buf, state);
            }
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

impl SelectAction<Task> for TodoWidget {
    fn get_selected_bf(
        &mut self,
        // current_target: &Option<Rc<RefCell<Task>>>,
        // targets: &Vec<Rc<RefCell<Task>>>,
        // state: &mut ListState,
        bf: super::SelectBF,
    ) -> Option<Rc<RefCell<Task>>> {
        if let Some(cur_list) = &self.current_todolist {
            if self.search_string.is_empty() {
                let task_list = TodoWidget::get_flattened(&cur_list.borrow().tasks);
                if !task_list.is_empty() {
                    let mut cur_list_mut = cur_list.borrow_mut();
                    if let Some(cur_task) = &cur_list_mut.current_task {
                        let (mut target, _) = task_list
                            .iter()
                            .enumerate()
                            .find(|(_, task)| task.borrow().id == cur_task.borrow().id)
                            .unwrap();
                        match bf {
                            SelectBF::Forward => {
                                target = (target + 1).min(task_list.len() - 1);
                                cur_list_mut.state.select(Some(target));
                                return Some(task_list[target].to_owned());
                            }
                            SelectBF::Back => {
                                target = target.saturating_sub(1);
                                cur_list_mut.state.select(Some(target));
                                return Some(task_list[target].to_owned());
                            }
                        }
                    } else {
                        match bf {
                            SelectBF::Forward => {
                                cur_list_mut.state.select(Some(0));
                                return Some(task_list[0].to_owned());
                            }
                            SelectBF::Back => {
                                cur_list_mut.state.select(Some(task_list.len() - 1));
                                return Some(task_list[task_list.len() - 1].to_owned());
                            }
                        }
                    }
                } else {
                    return None;
                }
            } else {
                let mut task_list = Vec::new();
                cur_list.borrow().tasks.iter().for_each(|task| {
                    if task.borrow().is_target(self.search_string.clone()) {
                        task_list.push(task.to_owned());
                    }
                });
                let tar_list = TodoWidget::get_flattened(&task_list);
                if !tar_list.is_empty() {
                    let mut cur_list_mut = cur_list.borrow_mut();
                    if let Some(cur_task) = &cur_list_mut.current_task {
                        let find_result = tar_list
                            .iter()
                            .enumerate()
                            .find(|(_, task)| task.borrow().id == cur_task.borrow().id);
                        if let Some((mut target, _)) = find_result {
                            match bf {
                                SelectBF::Forward => {
                                    target = (target + 1).min(tar_list.len() - 1);
                                    cur_list_mut.state.select(Some(target));
                                    return Some(tar_list[target].to_owned());
                                }
                                SelectBF::Back => {
                                    target = target.saturating_sub(1);
                                    cur_list_mut.state.select(Some(target));
                                    return Some(tar_list[target].to_owned());
                                }
                            }
                        } else {
                            match bf {
                                SelectBF::Forward => {
                                    cur_list_mut.state.select(Some(0));
                                    return Some(tar_list[0].to_owned());
                                }
                                SelectBF::Back => {
                                    cur_list_mut.state.select(Some(tar_list.len() - 1));
                                    return Some(tar_list[tar_list.len() - 1].to_owned());
                                }
                            }
                        }
                    } else {
                        match bf {
                            SelectBF::Forward => {
                                cur_list_mut.state.select(Some(0));
                                return Some(tar_list[0].to_owned());
                            }
                            SelectBF::Back => {
                                cur_list_mut.state.select(Some(tar_list.len() - 1));
                                return Some(tar_list[tar_list.len() - 1].to_owned());
                            }
                        }
                    }
                } else {
                    return None;
                }
            }
        }

        None
    }

    fn get_flattened(target: &Vec<Rc<RefCell<Task>>>) -> Vec<Rc<RefCell<Task>>> {
        let mut result = Vec::<Rc<RefCell<Task>>>::new();
        target.iter().for_each(|task| {
            result.push(task.clone());
            let task_bor = task.borrow();
            if !task_bor.children.is_empty() {
                let child = TodoWidget::get_flattened(&task_bor.children);
                result.extend(child);
            }
        });

        result
    }
}
