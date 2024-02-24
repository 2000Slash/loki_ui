use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Layout, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use super::Screen;

#[derive(Default)]
enum Selection {
    Cancel,
    #[default]
    Ok,
}

pub struct Alert<F>
where
    F: FnMut(),
{
    should_close: bool,
    selection: Selection,
    text: &'static str,
    title: &'static str,
    action: Option<F>,
}

impl<F> Alert<F>
where
    F: FnMut(),
{
    pub fn with_action(title: &'static str, text: &'static str, action: F) -> Self {
        Alert {
            title,
            text,
            action: Some(action),
            should_close: false,
            selection: Selection::Cancel,
        }
    }

    pub fn new(title: &'static str, text: &'static str) -> Self {
        Alert {
            title,
            text,
            action: None,
            should_close: false,
            selection: Selection::Cancel,
        }
    }

    fn bottom_buttons(&self, frame: &mut Frame, rect: Rect) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(
                [
                    ratatui::layout::Constraint::Max(1),
                    ratatui::layout::Constraint::Min(8),
                    ratatui::layout::Constraint::Max(1),
                    ratatui::layout::Constraint::Min(8),
                    ratatui::layout::Constraint::Max(1),
                ]
                .as_ref(),
            )
            .split(rect);

        self.confirm_button(frame, layout[1]);
        self.cancel_button(frame, layout[3]);
    }

    fn confirm_button(&self, frame: &mut Frame, rect: Rect) {
        let color = match self.selection {
            Selection::Ok => ratatui::style::Color::Blue,
            _ => ratatui::style::Color::White,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(color));

        frame.render_widget(
            Paragraph::new("Ok")
                .alignment(Alignment::Center)
                .block(block),
            rect,
        );
    }

    fn cancel_button(&self, frame: &mut Frame, rect: Rect) {
        let color = match self.selection {
            Selection::Cancel => ratatui::style::Color::Blue,
            _ => ratatui::style::Color::White,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(color));

        frame.render_widget(
            Paragraph::new("Cancel")
                .alignment(Alignment::Center)
                .block(block),
            rect,
        );
    }
}

impl<F> Screen for Alert<F>
where
    F: FnMut(),
{
    fn should_close(&self) -> bool {
        self.should_close
    }

    fn render(&mut self, frame: &mut ratatui::prelude::Frame, _app: &crate::ui::App) {
        let mut size = frame.size();
        size.x = size.width / 2 - 25;
        size.y = size.height / 2 - 5;
        size.width = 50;
        size.height = 10;
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    ratatui::layout::Constraint::Percentage(100),
                    ratatui::layout::Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(size);

        let block = Block::default().title(self.title).borders(Borders::ALL);

        frame.render_widget(Clear, block.inner(size));
        frame.render_widget(block, size);

        frame.render_widget(Paragraph::new(self.text), layout[0]);
        self.bottom_buttons(frame, layout[1]);
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent, _app: &mut crate::ui::App) {
        match key.code {
            KeyCode::Enter => match self.selection {
                Selection::Cancel => {
                    self.should_close = true;
                }
                Selection::Ok => {
                    self.should_close = true;
                    if let Some(action) = &mut self.action {
                        action();
                    }
                }
            },
            KeyCode::Right => {
                self.selection = Selection::Cancel;
            }
            KeyCode::Left => {
                self.selection = Selection::Ok;
            }
            _ => {}
        }
    }
}
