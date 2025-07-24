use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Styled, Stylize},
    text::Line,
    widgets::Widget,
};

use crate::app::appstate::CurrentFocus;

#[derive(Debug)]
pub struct KeyMap {
    pub focus: CurrentFocus,
    pub workspace_hint: String,
    pub tasklist_hint: String,
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
            workspace_hint: vec!["<a>add", "<x>delete", "<i>subworkspace", "<h>help"].join(" "),
            tasklist_hint: vec![
                "<a>add ",
                "<x>delete ",
                "<i>subtask ",
                "<c>complete ",
                "<p>inprocess",
                "<t>todo",
                //TODO: Implement rename functionality
                "<r>rename ",
                //TODO: Implement sort functionality
                "<s>sort ",
                //TODO: Implement filter functionality
                "<f>filter ",
                "<h>help ",
            ]
            .join(" "),
        }
    }
}

impl Widget for &mut KeyMap {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render the key map widget
        let keymap_hint: String = match self.focus {
            CurrentFocus::TodoList => self.tasklist_hint.clone(),
            CurrentFocus::Workspace => self.workspace_hint.clone(),
        };
        let hint_line = Line::from(keymap_hint).fg(Color::White);
        Widget::render(hint_line, area, buf);
    }
}
