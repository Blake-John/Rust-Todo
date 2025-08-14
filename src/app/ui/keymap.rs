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
}

impl Keymap {
    fn new(key: &str, desc: &str) -> Self {
        Self {
            key: key.to_string(),
            desc: desc.to_string(),
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
                Keymap::new("h/left", "move left"),
                Keymap::new("l/right", "move right"),
                Keymap::new("j/down", "move down"),
                Keymap::new("k/up", "move up"),
                Keymap::new("tab", "change focus"),
                Keymap::new("enter", "enter workspace"),
                Keymap::new("esc", "exit current mode"),
                Keymap::new("q", "exit application"),
            ],
            workspace_hint: vec![
                Keymap::new("a", "add"),
                Keymap::new("x", "delete"),
                Keymap::new("i", "subworkspace"),
                Keymap::new("A", "archive"),
                Keymap::new("r", "rename"),
                Keymap::new("?", "help"),
            ],
            tasklist_hint: vec![
                Keymap::new("a", "add"),
                Keymap::new("x", "delete"),
                Keymap::new("i", "subtask"),
                Keymap::new("c", "complete"),
                Keymap::new("p", "inprocess"),
                Keymap::new("t", "todo"),
                Keymap::new("r", "rename"),
                //TODO: Implement sort functionality
                Keymap::new("s", "sort"),
                Keymap::new("f /", "filter"),
                // TODO: Implement the help popup window
                Keymap::new("?", "help"),
            ],
            archived_ws_hint: vec![
                Keymap::new("x", "delete"),
                Keymap::new("r", "rename"),
                Keymap::new("R", "recovery"),
                Keymap::new("?", "help"),
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
