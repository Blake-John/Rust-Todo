use std::{cell::RefCell, rc::Rc};

use ratatui::{
    layout::{Constraint, Layout},
    widgets::{Block, ScrollbarState, Widget},
};

use crate::app::ui::keymap::KeymapWidget;

#[derive(Debug, Default)]
pub struct HelpWidget {
    pub scroll: usize,
    pub state: ScrollbarState,
    pub keymap: Rc<RefCell<KeymapWidget>>,
}

impl HelpWidget {
    fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Widget for &mut HelpWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let v_layouts = Layout::vertical([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(area);
        let h_layout = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(v_layouts[1]);

        let block = Block::bordered()
            .title("Help Page")
            .title_alignment(ratatui::layout::Alignment::Center);
    }
}
