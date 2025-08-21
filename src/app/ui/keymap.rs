use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Widget,
};

use crate::app::appstate::{CurrentFocus, CurrentMode};

#[derive(Debug)]
pub struct Keymap {
    pub key: String,
    pub desc: String,
    pub detailed: String,
}

impl Keymap {
    fn new(key: &str, desc: &str, detail: &str) -> Self {
        Self {
            key: key.to_string(),
            desc: desc.to_string(),
            detailed: detail.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct KeymapWidget {
    pub focus: CurrentFocus,
    pub mode: CurrentMode,
    pub general_hint: Vec<Keymap>,
    pub workspace_hint: Vec<Keymap>,
    pub tasklist_hint: Vec<Keymap>,
    pub archived_ws_hint: Vec<Keymap>,
}

impl KeymapWidget {
    pub fn new(focus: CurrentFocus) -> Self {
        KeymapWidget {
            focus,
            ..Default::default()
        }
    }
}

impl Default for KeymapWidget {
    fn default() -> Self {
        KeymapWidget {
            focus: CurrentFocus::Workspace,
            mode: CurrentMode::Normal,
            general_hint: vec![
                Keymap::new("h/left", "left", "focus on left part(workspace)"),
                Keymap::new("l/right", "right", "focus on right part(tasks)"),
                Keymap::new("j/down", "down", "select item bellow"),
                Keymap::new("k/up", "up", "select item above"),
                Keymap::new("tab", "focus", "change focus between 3 parts"),
                Keymap::new(
                    "enter",
                    "enter workspace",
                    "enter into the tasks of the workspace",
                ),
                Keymap::new("esc", "exit current mode", "exit search or help"),
                Keymap::new("q", "quit", "quit the application"),
                Keymap::new("ctrl-s", "save", "save the data"),
                Keymap::new("1/2/3", "focus", "focus target part"),
            ],
            workspace_hint: vec![
                Keymap::new("a", "add", "add new workspace"),
                Keymap::new("x", "delete", "delete current workspace"),
                Keymap::new("i", "subworkspace", "insert a subworkspace to current"),
                Keymap::new("A", "archive", "archive current workspace"),
                Keymap::new("r", "rename", "rename current workspace"),
                Keymap::new("ctrl-s", "save", "save the data"),
                Keymap::new("?", "help", "open the help page"),
            ],
            tasklist_hint: vec![
                Keymap::new("a", "add", "add new task"),
                Keymap::new("x", "delete", "delete current task"),
                Keymap::new("i", "subtask", "insert a subtask to current"),
                Keymap::new("c", "complete", "mark the task as completed"),
                Keymap::new("p", "inprocess", "mark the task as in process"),
                Keymap::new("t", "todo", "mark the task as todo"),
                Keymap::new("d", "deprecate", "mark the task as deprecated"),
                Keymap::new("D", "due", "set the due date of current task"),
                Keymap::new("r", "rename", "rename the current task"),
                //TODO: Implement sort functionality
                Keymap::new("s", "sort", "sort the current task by rule (in dev)"),
                Keymap::new("f /", "filter", "search tasks in current workspace"),
                Keymap::new("+/=", "increase", "increase the urgency"),
                Keymap::new("-/_", "decrease", "decrease the urgency"),
                Keymap::new("ctrl-s", "save", "save the data"),
                Keymap::new("?", "help", "open the help page"),
            ],
            archived_ws_hint: vec![
                Keymap::new("x", "delete", "delete current workspace"),
                Keymap::new("r", "rename", "rename current workspace"),
                Keymap::new("R", "recovery", "recovery the current workspace"),
                Keymap::new("ctrl-s", "save", "save the data"),
                Keymap::new("?", "help", "open the help page"),
            ],
        }
    }
}

impl Widget for &mut KeymapWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render the key map widget
        let mut hint_span: Vec<Span> = Vec::new();
        if let CurrentMode::Help = &self.mode {
            self.general_hint.iter().for_each(|hint| {
                hint_span.push(Span::styled(" ".to_string(), Style::new().white()));
                hint_span.push(Span::styled("<".to_string(), Style::new().white()));
                hint_span.push(Span::styled(hint.key.clone(), Style::new().light_cyan()));
                hint_span.push(Span::styled(">".to_string(), Style::new().white()));
                hint_span.push(Span::styled(hint.desc.clone(), Style::new().white()));
                hint_span.push(Span::styled(" ".to_string(), Style::new().white()));
            });
        } else {
            match self.focus {
                CurrentFocus::TodoList => {
                    self.tasklist_hint.iter().for_each(|hint| {
                        hint_span.push(Span::styled(" ".to_string(), Style::new().white()));
                        hint_span.push(Span::styled("<".to_string(), Style::new().white()));
                        hint_span.push(Span::styled(hint.key.clone(), Style::new().light_blue()));
                        hint_span.push(Span::styled(">".to_string(), Style::new().white()));
                        hint_span.push(Span::styled(hint.desc.clone(), Style::new().white()));
                        hint_span.push(Span::styled(" ".to_string(), Style::new().white()));
                    });
                }
                CurrentFocus::Workspace => {
                    self.workspace_hint.iter().for_each(|hint| {
                        hint_span.push(Span::styled(" ".to_string(), Style::new().white()));
                        hint_span.push(Span::styled("<".to_string(), Style::new().white()));
                        hint_span.push(Span::styled(hint.key.clone(), Style::new().light_green()));
                        hint_span.push(Span::styled(">".to_string(), Style::new().white()));
                        hint_span.push(Span::styled(hint.desc.clone(), Style::new().white()));
                        hint_span.push(Span::styled(" ".to_string(), Style::new().white()));
                    });
                }
                CurrentFocus::ArchivedWorkspace => {
                    self.archived_ws_hint.iter().for_each(|hint| {
                        hint_span.push(Span::styled(" ".to_string(), Style::new().white()));
                        hint_span.push(Span::styled("<".to_string(), Style::new().white()));
                        hint_span.push(Span::styled(hint.key.clone(), Style::new().light_yellow()));
                        hint_span.push(Span::styled(">".to_string(), Style::new().white()));
                        hint_span.push(Span::styled(hint.desc.clone(), Style::new().white()));
                        hint_span.push(Span::styled(" ".to_string(), Style::new().white()));
                    });
                }
            };
        }

        let hint_line = Line::from(hint_span);
        Widget::render(hint_line, area, buf);
    }
}
