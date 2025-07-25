use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Widget,
};

use crate::app::appstate::CurrentFocus;

#[derive(Debug)]
pub struct KeyMap {
    pub focus: CurrentFocus,
    pub workspace_hint: Vec<&'static str>,
    pub tasklist_hint: Vec<&'static str>,
}

impl KeyMap {
    pub fn new(focus: CurrentFocus) -> Self {
        KeyMap {
            focus,
            ..Default::default()
        }
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        KeyMap {
            focus: CurrentFocus::Workspace,
            workspace_hint: vec!["<a>add ", "<x>delete ", "<i>subworkspace ", "<h>help "],
            tasklist_hint: vec![
                "<a>add ",
                "<x>delete ",
                "<i>subtask ",
                "<c>complete ",
                "<p>inprocess ",
                "<t>todo ",
                //TODO: Implement rename functionality
                "<r>rename ",
                //TODO: Implement sort functionality
                "<s>sort ",
                //TODO: Implement filter functionality
                "<f>filter ",
                "<h>help ",
            ],
        }
    }
}

impl Widget for &mut KeyMap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render the key map widget
        let keymap_hint: &Vec<&str> = match self.focus {
            CurrentFocus::TodoList => &self.tasklist_hint,
            CurrentFocus::Workspace => &self.workspace_hint,
        };
        let mut hint_span: Vec<Span> = Vec::new();
        keymap_hint.iter().for_each(|hi| {
            match self.focus {
                CurrentFocus::TodoList => {
                    hi.chars().enumerate().for_each(|(i, c)| {
                        let style = if i == 1 {
                            Style::new().blue()
                        } else {
                            Style::new().white()
                        };
                        hint_span.push(Span::styled(c.to_string(), style));
                    });
                }
                CurrentFocus::Workspace => {
                    hi.chars().enumerate().for_each(|(i, c)| {
                        let style = if i == 1 {
                            Style::new().green()
                        } else {
                            Style::new().white()
                        };
                        hint_span.push(Span::styled(c.to_string(), style));
                    });
                }
            };
        });
        let hint_line = Line::from(hint_span);
        Widget::render(hint_line, area, buf);
    }
}
