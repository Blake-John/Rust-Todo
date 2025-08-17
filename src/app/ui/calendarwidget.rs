use std::vec;

use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Row, Table, Widget},
};

use crate::app::ui::Ui;

#[derive(Debug)]
pub struct CalendarWidget {
    pub today: NaiveDate,
    pub cursor: NaiveDate,
}

impl CalendarWidget {
    pub fn new() -> Self {
        let today = Local::now().date_naive();
        Self {
            today,
            cursor: today,
        }
    }

    pub fn move_up(&mut self) {
        for _ in 0..7 {
            self.cursor = self.cursor.pred_opt().unwrap_or(self.cursor);
        }
    }
    pub fn move_down(&mut self) {
        for _ in 0..7 {
            self.cursor = self.cursor.succ_opt().unwrap_or(self.cursor);
        }
    }
    pub fn move_left(&mut self) {
        self.cursor = self.cursor.pred_opt().unwrap_or(self.cursor);
    }
    pub fn move_right(&mut self) {
        self.cursor = self.cursor.succ_opt().unwrap_or(self.cursor);
    }
    pub fn same_month(&self) -> bool {
        self.cursor.month() == self.today.month() && self.cursor.year() == self.today.year()
    }
}

impl Default for CalendarWidget {
    fn default() -> Self {
        Self::new()
    }
}

fn get_calendar_window(area: Rect) -> Rect {
    let layout1 = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(14 * 2),
        Constraint::Fill(1),
    ])
    .split(area);

    Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(6 + 1 + 1 + 2),
        Constraint::Fill(1),
    ])
    .split(layout1[1])[1]
}

impl Widget for &mut CalendarWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let center_layout = get_calendar_window(area);
        let layouts = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)])
            .margin(1)
            .split(center_layout);
        let block = Block::bordered().title(Line::from(" Calendar ").centered());

        Widget::render(Clear, center_layout, buf);
        Widget::render(block, center_layout, buf);

        let mut day_rows = vec![];
        let first_day_of_month =
            NaiveDate::from_ymd_opt(self.cursor.year(), self.cursor.month(), 1).unwrap();
        let mut day = first_day_of_month
            - Duration::days(first_day_of_month.weekday().num_days_from_monday() as i64);
        for _week in 0..6 {
            let mut cells = vec![];
            for _day in 0..7 {
                let is_today = day == self.today;
                let is_cursor = day == self.cursor;
                let mut style = Style::default();

                if day.month() != self.cursor.month() {
                    style = style.fg(Color::DarkGray);
                }

                if is_today {
                    style = style.fg(Color::Yellow);
                }

                if is_cursor {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                cells.push(Span::styled(format!("{:2}", day.day()), style));
                day = day.succ_opt().unwrap();
            }
            day_rows.push(Row::new(cells));
        }
        let header = Row::new(vec!["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]).green();
        let table = Table::new(day_rows, [Constraint::Length(4); 7])
            .header(header)
            .column_spacing(1);

        let year_month_label = Line::from(vec![
            Span::from(format!("{}", self.cursor.year())).fg(Color::LightRed),
            Span::from("  "),
            Span::from(format!("{}", self.cursor.month())).fg(Color::LightRed),
        ]);

        Widget::render(year_month_label, layouts[0], buf);
        Widget::render(table, layouts[1], buf);
    }
}
