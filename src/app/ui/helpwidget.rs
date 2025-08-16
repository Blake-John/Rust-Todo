use ratatui::{
    layout::{Constraint, Layout},
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Scrollbar, ScrollbarState, StatefulWidget, Widget},
};

use crate::app::ui::keymap::KeymapWidget;

#[derive(Debug, Default)]
pub struct HelpWidget {
    pub scroll: usize,
    pub scroll_max: usize,
    pub state: ScrollbarState,
    pub keymap: KeymapWidget,
}

impl HelpWidget {
    pub fn new() -> Self {
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
            .title(" Help Page ")
            // .border_style(Style::new().light_cyan())
            .title_alignment(ratatui::layout::Alignment::Center);

        let keymap_widget = &self.keymap;
        let mut general_keys: Vec<Line> = vec![Line::from("General Keymaps").bold().light_cyan()];
        let mut workspace_keys: Vec<Line> =
            vec![Line::from("Workspace Keymaps").bold().light_green()];
        let mut todolist_keys: Vec<Line> =
            vec![Line::from("Todo List Keymaps").bold().light_blue()];
        let mut archived_ws_keys: Vec<Line> = vec![
            Line::from("Archived Workspace Keymaps")
                .bold()
                .light_yellow(),
        ];

        keymap_widget.general_hint.iter().for_each(|hint| {
            general_keys.push(Line::from(vec![
                Span::from(format!("{:12}", hint.key.to_owned())).light_cyan(),
                Span::from(format!("{:12}", hint.detailed.to_owned())),
            ]));
        });
        general_keys.push(Line::from(""));
        keymap_widget.workspace_hint.iter().for_each(|hint| {
            workspace_keys.push(Line::from(vec![
                Span::from(format!("{:12}", hint.key.to_owned())).light_green(),
                Span::from(format!("{:12}", hint.detailed.to_owned())),
            ]));
        });
        workspace_keys.push(Line::from(""));
        keymap_widget.tasklist_hint.iter().for_each(|hint| {
            todolist_keys.push(Line::from(vec![
                Span::from(format!("{:12}", hint.key.to_owned())).light_blue(),
                Span::from(format!("{:12}", hint.detailed.to_owned())),
            ]));
        });
        todolist_keys.push(Line::from(""));
        keymap_widget.archived_ws_hint.iter().for_each(|hint| {
            archived_ws_keys.push(Line::from(vec![
                Span::from(format!("{:12}", hint.key.to_owned())).light_yellow(),
                Span::from(format!("{:12}", hint.detailed.to_owned())),
            ]));
        });
        archived_ws_keys.push(Line::from(""));

        let mut para_lines = Vec::new();
        para_lines.extend(general_keys);
        para_lines.extend(workspace_keys);
        para_lines.extend(todolist_keys);
        para_lines.extend(archived_ws_keys);
        self.state = self.state.content_length(para_lines.len() - 5 - 1);
        self.scroll_max = para_lines.len() - 5 - 1;
        let para = Paragraph::new(para_lines)
            .block(block)
            .scroll((self.scroll as u16, 0));
        let scrollbar = Scrollbar::new(ratatui::widgets::ScrollbarOrientation::VerticalRight);

        Widget::render(Clear, h_layout[1], buf);
        Widget::render(para, h_layout[1], buf);
        StatefulWidget::render(scrollbar, h_layout[1], buf, &mut self.state);
    }
}
