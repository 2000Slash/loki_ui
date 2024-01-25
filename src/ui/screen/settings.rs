use ratatui::{
    layout::{Layout, Rect},
    style::Style,
    widgets::{Block, Borders, Clear},
};
use tui_textarea::TextArea;

use crate::ui::App;

use super::Screen;

enum Selection {
    Url,
    None,
}

pub struct Settings<'a> {
    loki_url: TextArea<'a>,
    selection: Selection,
    should_close: bool,
}

impl Settings<'_> {
    pub fn new(url: String) -> Self {
        let mut loki_url = TextArea::new(vec![url]);
        loki_url.set_cursor_line_style(Style::default());
        Self {
            loki_url,
            selection: Selection::None,
            should_close: false,
        }
    }
}

impl Screen for Settings<'_> {
    fn should_close(&self) -> bool {
        self.should_close
    }

    fn render(&self, frame: &mut ratatui::prelude::Frame, _app: &crate::ui::App) {
        let settings_window_block = Block::default().title("Settings").borders(Borders::ALL);
        let mut settings_window_size = Rect::default();
        settings_window_size.width = 60;
        settings_window_size.height = 6;
        settings_window_size.x = (frame.size().width / 2) - (settings_window_size.width / 2);
        settings_window_size.y = (frame.size().height / 2) - (settings_window_size.height / 2);
        let inner_size = settings_window_block.inner(settings_window_size);
        frame.render_widget(Clear, inner_size);
        frame.render_widget(settings_window_block, settings_window_size);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Percentage(100),
            ])
            .split(inner_size);

        let style = match self.selection {
            Selection::Url => ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
            Selection::None => ratatui::style::Style::default().fg(ratatui::style::Color::Blue),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Loki URL")
            .style(style);
        frame.render_widget(self.loki_url.widget(), block.inner(layout[1]));
        frame.render_widget(block, layout[1]);
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent, app: &mut App) {
        match self.selection {
            Selection::Url => match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.selection = Selection::None;
                }
                crossterm::event::KeyCode::Enter => {
                    self.selection = Selection::None;
                }
                _ => {
                    self.loki_url.input(key);
                }
            },
            Selection::None => match key.code {
                crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('q') => {
                    app.config.loki_url = self.loki_url.lines()[0].to_string();
                    self.should_close = true;
                }
                crossterm::event::KeyCode::Enter => {
                    self.selection = Selection::Url;
                }
                _ => {}
            },
        }
    }
}
