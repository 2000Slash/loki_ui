use std::{collections::HashMap, sync::{Arc, Mutex}};

use crossterm::event::KeyEvent;
use log::{error, info};
use ratatui::{layout::{Layout, Rect}, style::{Color, Style}, text::{Line, Span, Text}, widgets::Paragraph, Frame};

#[cfg(feature = "debug")]
use tui_logger::{TuiLoggerSmartWidget, TuiLoggerLevelOutput};
use tui_textarea::TextArea;

use crate::{loki::Loki, ui::{App, Store}};

use super::Screen;

use ratatui::widgets::{Block, Borders};


enum Selection {
    Query(bool),
    Results,
}

pub struct Query<'a> {
    query_textarea: TextArea<'a>,
    selection: Selection,
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
            query_textarea: textarea,
            selection: Selection::Query(false),
        }
    }

    /// Draws the bottom Keyboard hints row
    fn draw_keyhints(frame: &mut Frame, rect: Rect) {
        let mut keymap: HashMap<char, String> = HashMap::new();
        keymap.insert('Q', String::from("Quit"));

        let mut text = Line::from("");
        for (key, value) in keymap {
            for char in value.chars() {
                let mut style = Style::default().fg(Color::Gray);
                if char == key {
                    style = style.fg(Color::Red);
                }
                text.spans.push(Span::styled(String::from(char), style));
            }
        }
        frame.render_widget(Paragraph::new(text), rect);
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

    fn results_frame(&self, frame: &mut Frame, rect: Rect, app: &App) {
        let color = match self.selection {
            Selection::Results => ratatui::style::Color::Blue,
            _ => ratatui::style::Color::White,
        };

        let block =             Block::default()
        .title("Results")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));

        let store = app.store.lock().unwrap();

        frame.render_widget(
                Paragraph::new(Text::from(store.results.join("\n")))
                    .block(block),
                rect,
        );
    }
}

impl Screen for Query<'_> {
    fn render(&self, frame: &mut ratatui::prelude::Frame, app: &App) {
        let layout = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .margin(1)
        .constraints(
            [
                ratatui::layout::Constraint::Length(3),
                ratatui::layout::Constraint::Percentage(50),
                #[cfg(feature = "debug")]
                ratatui::layout::Constraint::Percentage(50)
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
        self.results_frame(frame, layout[1], app);

        let height = frame.size().height;
        let offset = 3;
        Query::draw_keyhints(frame, Rect::new(offset, height - 1, frame.size().width - offset, 1));

        #[cfg(feature = "debug")]
        frame.render_widget(
            TuiLoggerSmartWidget::default()
                        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Magenta))
        .style_info(Style::default().fg(Color::Cyan))
        .output_separator(':')
        .output_timestamp(Some("%H:%M:%S".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        .output_target(true)
        .output_file(true)
        .output_line(true),
            layout[2]
        );
    }

    fn handle_key_event(&mut self, key: KeyEvent, loki: &mut Loki, store: Arc<Mutex<Store>>) -> bool {
        match self.selection {
            Selection::Query(true) => {
                match key.code {
                    crossterm::event::KeyCode::Esc => {
                        self.selection = Selection::Query(false);
                        return true;
                    }
                    crossterm::event::KeyCode::Enter => {
                        self.selection = Selection::Results;
                        let text = self.query_textarea.lines()[0].to_string();
                        let mut loki = loki.clone();
                        tokio::spawn(async move {
                            let result = loki.query_range(&text, None, None, None).await;
                            info!("{:?}", result);
                            let mut store = store.lock().unwrap();
                            if let Ok(result) = result {
                                let results_text: Vec<String> = result.iter().map(|result| {
                                    let mut string = String::new();
                                    string.push_str(&format!("Labels: {:?}\n", result.labels));
                                    string.push_str("Values:\n");
                                    for value in &result.values {
                                        string.push_str(&format!("  {:?}\n", value));
                                    }
                                    string
                                }).collect();
                                store.results = results_text;
                            } else {
                                let error = result.unwrap_err();
                                store.results = vec!["No results".to_string(), error.to_string()];
                            }
                        });
                        return true;
                    }
                    _ => {
                        return self.query_textarea.input(key);
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
