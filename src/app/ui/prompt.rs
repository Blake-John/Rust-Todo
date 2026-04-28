use ratatui::{style::Stylize, text::Line, widgets::Widget};

use crate::app::appstate::CurrentMode;

#[derive(Debug)]
pub struct PromptWidget {
    pub padding: String,
    pub desc: String,
}

impl PromptWidget {
    pub fn new() -> Self {
        Self {
            padding: String::from("  "),
            desc: String::from("In Normal Mode"),
        }
    }

    pub fn update_mode(&mut self, mode: CurrentMode) {
        self.desc = match mode {
            CurrentMode::Normal => "In Normal Mode".to_string(),
            CurrentMode::Insert => "In Insert Mode".to_string(),
            CurrentMode::Search => "In Search Mode".to_string(),
            CurrentMode::Help => "In Help Mode".to_string(),
            CurrentMode::Sort => "In Sort Mode".to_string(),
        };
    }
}

impl Default for PromptWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for &mut PromptWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        Widget::render(
            Line::from(vec![
                self.padding.clone().into(),
                self.desc.clone().light_cyan(),
            ]),
            area,
            buf,
        );
    }
}
