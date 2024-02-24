use std::thread;

use ratatui::layout::{Alignment, Layout, Rect};
use ratatui::prelude::Style;
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;
use tui_textarea::TextArea;

use super::alert::Alert;
use super::Screen;

enum Buttons {
    Left,
    Right,
}

enum Selection {
    Query(bool),
    Buttons(Buttons),
}

pub struct Remove<'a> {
    should_close: bool,
    query_textarea: TextArea<'a>,
    selection: Selection,
}

impl Default for Remove<'_> {
    fn default() -> Self {
        Remove::new(&[])
    }
}

impl Remove<'_> {
    fn query_bar(&self, frame: &mut Frame, rect: Rect) {
        let color = match self.selection {
            Selection::Query(true) => ratatui::style::Color::Yellow,
            Selection::Query(false) => ratatui::style::Color::Blue,
            _ => ratatui::style::Color::White,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title("Query")
            .border_style(Style::default().fg(color));
        let inner_area = block.inner(rect);
        frame.render_widget(block, rect);
        frame.render_widget(self.query_textarea.widget(), inner_area);
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

        self.remove_button(frame, layout[1]);
        self.cancel_button(frame, layout[3]);
    }

    fn remove_button(&self, frame: &mut Frame, rect: Rect) {
        let color = match self.selection {
            Selection::Buttons(Buttons::Left) => ratatui::style::Color::Blue,
            _ => ratatui::style::Color::White,
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::default().fg(color));

        frame.render_widget(
            Paragraph::new("Delete")
                .alignment(Alignment::Center)
                .block(block),
            rect,
        );
    }

    fn cancel_button(&self, frame: &mut Frame, rect: Rect) {
        let color = match self.selection {
            Selection::Buttons(Buttons::Right) => ratatui::style::Color::Blue,
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

    pub fn new(query: &[String]) -> Self {
        let mut query_textarea = TextArea::new(query.to_owned());
        query_textarea.set_cursor_line_style(Style::default());
        query_textarea.set_placeholder_text("Enter a valid query");
        Remove {
            should_close: false,
            query_textarea,
            selection: Selection::Query(false),
        }
    }
}

impl Screen for Remove<'_> {
    fn should_close(&self) -> bool {
        self.should_close
    }

    fn render(&mut self, frame: &mut ratatui::prelude::Frame, _app: &crate::ui::App) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    ratatui::layout::Constraint::Length(3),
                    ratatui::layout::Constraint::Percentage(100),
                    ratatui::layout::Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(frame.size());

        frame.render_widget(Clear, frame.size());
        frame.render_widget(
            Block::default().title("Remove").borders(Borders::ALL),
            frame.size(),
        );

        self.query_bar(frame, layout[0]);
        self.bottom_buttons(frame, layout[2]);
        //self.results_frame(frame, layout[1], app);
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent, app: &mut crate::ui::App) {
        match self.selection {
            Selection::Query(true) => match key.code {
                crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Enter => {
                    self.selection = Selection::Query(false);
                }
                _ => {
                    self.query_textarea.input(key);
                }
            },
            _ => match key.code {
                crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                    self.should_close = true;
                }
                crossterm::event::KeyCode::Down => {
                    self.selection = Selection::Buttons(Buttons::Right);
                }
                crossterm::event::KeyCode::Up => {
                    self.selection = Selection::Query(false);
                }
                crossterm::event::KeyCode::Left => {
                    if let Selection::Buttons(_) = self.selection {
                        self.selection = Selection::Buttons(Buttons::Left);
                    }
                }
                crossterm::event::KeyCode::Enter => match self.selection {
                    Selection::Query(false) => {
                        self.selection = Selection::Query(true);
                    }
                    Selection::Buttons(Buttons::Right) => {
                        self.should_close = true;
                    }
                    Selection::Buttons(Buttons::Left) => {
                        let loki = app.loki.clone();
                        let query = self.query_textarea.lines()[0].to_string();
                        app.screens
                            .push(Box::from(Alert::with_action("Remove data for query?", "This will create a delete request for all Data\nfound with this Query. Once the delete request\nis executed, you cant undo this!", move || {
                                let loki = loki.clone();
                                let query = query.clone();
                                thread::spawn(move || {
                                    let mut loki = loki.clone();
                                    loki.delete(&query, None, None).unwrap();
                                });
                            })));
                    }
                    _ => (),
                },
                crossterm::event::KeyCode::Right => {
                    if let Selection::Buttons(_) = self.selection {
                        self.selection = Selection::Buttons(Buttons::Right);
                    }
                }
                _ => {}
            },
        }
    }
}
