use crossterm::event::KeyEvent;
use ratatui::{layout::{Layout, Rect}, Frame, style::Style};
use tui_textarea::TextArea;

use super::Screen;

use ratatui::widgets::{Block, Borders};


enum Selection {
    Query(bool),
    Results,
}

pub struct Query<'a> {
    query: String,
    query_textarea: TextArea<'a>,
    selection: Selection,
    results: Vec<String>,
}

impl Default for Query<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Query<'_> {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.set_placeholder_text("Enter a valid query");
        Self {
            query: String::new(),
            query_textarea: textarea,
            results: Vec::new(),
            selection: Selection::Query(false),
        }
    }


    fn query_bar(&self, frame: &mut Frame, rect: Rect) {
        let color = match self.selection {
            Selection::Query(true) => ratatui::style::Color::Yellow,
            Selection::Query(false) => ratatui::style::Color::Blue,
            Selection::Results => ratatui::style::Color::White,
        };

        let block = Block::default().borders(Borders::ALL).title("Query").border_style(Style::default().fg(color));
        let inner_area = block.inner(rect);
        frame.render_widget(block, rect);
        frame.render_widget(self.query_textarea.widget(), inner_area);
    }

    fn results_frame(&self, frame: &mut Frame, rect: Rect) {
        let color = match self.selection {
            Selection::Results => ratatui::style::Color::Blue,
            _ => ratatui::style::Color::White,
        };

        frame.render_widget(
            Block::default()
                .title("Results")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color)),
                rect,
        );
    }
}

impl Screen for Query<'_> {
    fn render(&self, frame: &mut ratatui::prelude::Frame) {
        let layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .margin(1)
        .constraints(
            [
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Min(3),
            ]
            .as_ref(),
        )
        .split(frame.size());
    
        frame.render_widget(
            Block::default()
                .title("Loki Ui")
                .borders(Borders::ALL),
                frame.size(),
        );
    
        self.query_bar(frame, layout[0]);
        self.results_frame(frame, layout[1]);
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> bool {
        match self.selection {
            Selection::Query(true) => {
                match key.code {
                    crossterm::event::KeyCode::Esc => {
                        self.selection = Selection::Query(false);
                        return true;
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.selection = Selection::Results;
                        return true;
                    }
                    _ => {
                        self.query_textarea.input(key);
                    }
                }
            }
            Selection::Query(false) => {
                match key.code {
                    crossterm::event::KeyCode::Up => {
                        self.selection = Selection::Query(false);
                        return true;
                    }
                    crossterm::event::KeyCode::Down => {
                        self.selection = Selection::Results;
                        return true;
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.selection = Selection::Query(true);
                        return true;
                    }
                    _ => { }
                }
            }
            Selection::Results => {
                match key.code {
                    crossterm::event::KeyCode::Up => {
                        self.selection = Selection::Query(false);
                        return true;
                    }
                    crossterm::event::KeyCode::Down => {
                        self.selection = Selection::Results;
                        return true;
                    }
                    _ => { }
                }
            }
        }

        false
        
    }
}