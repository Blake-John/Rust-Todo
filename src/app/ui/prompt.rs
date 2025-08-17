use ratatui::{style::Stylize, text::Line, widgets::Widget};

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
