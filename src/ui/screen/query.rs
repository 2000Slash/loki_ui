use std::{collections::HashMap, thread, vec};

use crossterm::event::KeyEvent;
use log::info;

use ratatui::{
    layout::{Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

#[cfg(feature = "debug")]
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerSmartWidget};
use tui_textarea::TextArea;

use crate::ui::App;

use super::{remove::Remove, settings::Settings, Screen};

use ratatui::widgets::{Block, Borders};

#[derive(PartialEq)]
enum Selection {
    Query(bool),
    Results(bool),
}

pub struct Query<'a> {
    query_textarea: TextArea<'a>,
    results_textarea: TextArea<'a>,
    selection: Selection,
    should_close: bool,
}

impl Default for Query<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl Query<'_> {
    pub fn new() -> Self {
        let mut query_textarea = TextArea::default();
        query_textarea.set_cursor_line_style(Style::default());
        query_textarea.set_placeholder_text("Enter a valid query");
        let mut results_textarea = TextArea::default();
        results_textarea.set_cursor_line_style(Style::default());
        Self {
            results_textarea,
            query_textarea,
            selection: Selection::Query(false),
            should_close: false,
        }
    }

    /// Draws the bottom Keyboard hints row
    fn draw_keyhints(frame: &mut Frame, rect: Rect) {
        let mut keymap: HashMap<char, String> = HashMap::new();
        keymap.insert('q', String::from("quit"));
        keymap.insert('s', String::from("settings"));
        keymap.insert('d', String::from("delete"));
        // quick hack to get the keys in the right order
        let keys = vec!['q', 's', 'd'];

        let mut text = Line::from("");
        for key in keys {
            let value = keymap.get(&key).unwrap();
            let mut found = false;
            let mut style = Style::default().fg(Color::Gray);
            for char in value.chars() {
                if char == key && !found {
                    style = style.fg(Color::Red);
                    found = true;
                } else {
                    style = style.fg(Color::Gray);
                }
                text.spans.push(Span::styled(String::from(char), style));
            }
            text.spans.push(Span::raw(String::from("─")));
        }
        frame.render_widget(Paragraph::new(text), rect);
    }

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

    fn results_frame(&mut self, frame: &mut Frame, rect: Rect, app: &App) {
        let color = match self.selection {
            Selection::Results(false) => ratatui::style::Color::Blue,
            Selection::Results(true) => ratatui::style::Color::Yellow,
            _ => ratatui::style::Color::White,
        };

        let block = Block::default()
            .title("Results")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color));

        let mut store = app.store.lock().unwrap();
        if store.results_changed {
            self.results_textarea = TextArea::new(store.results.clone());
            self.results_textarea
                .set_cursor_line_style(Style::default());
            store.results_changed = false;
        }

        let inner_size = block.inner(rect);
        frame.render_widget(block, rect);
        frame.render_widget(self.results_textarea.widget(), inner_size);
    }
}

impl Screen for Query<'_> {
    fn should_close(&self) -> bool {
        self.should_close
    }

    fn render(&mut self, frame: &mut ratatui::prelude::Frame, app: &App) {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    ratatui::layout::Constraint::Length(3),
                    ratatui::layout::Constraint::Percentage(100),
                    #[cfg(feature = "debug")]
                    ratatui::layout::Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(frame.size());

        frame.render_widget(
            Block::default().title("Loki Ui").borders(Borders::ALL),
            frame.size(),
        );

        self.query_bar(frame, layout[0]);
        self.results_frame(frame, layout[1], app);

        let height = frame.size().height;
        let offset = 3;
        Query::draw_keyhints(
            frame,
            Rect::new(offset, height - 1, frame.size().width - offset, 1),
        );

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
            layout[2],
        );
    }

    fn handle_key_event(&mut self, key: KeyEvent, app: &mut App) {
        match self.selection {
            Selection::Query(true) => match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.selection = Selection::Query(false);
                }
                crossterm::event::KeyCode::Enter => {
                    self.selection = Selection::Results(false);
                    let text = self.query_textarea.lines()[0].to_string();
                    let mut loki = app.loki.clone();
                    let store = app.store.clone();
                    thread::spawn(move || {
                        let result = loki.query_range(&text, None, None, None);
                        info!("{:?}", result);
                        let mut store = store.lock().unwrap();
                        if let Ok(result) = result {
                            let results_text: Vec<String> = result
                                .iter()
                                .map(|result| {
                                    let mut string = String::new();
                                    string.push_str(&format!(
                                        "Labels: \n{}\n",
                                        serde_json::to_string_pretty(&result.labels).unwrap()
                                    ));
                                    string.push_str("\nValues:\n");
                                    for value in &result.values {
                                        string.push_str(&format!("  {}\n", value));
                                    }
                                    string.push('\n');
                                    string
                                })
                                .collect();
                            store.results = Vec::new();
                            for result in results_text {
                                for line in result.lines() {
                                    store.results.push(line.to_string());
                                }
                            }
                        } else {
                            let error = result.unwrap_err().to_string();
                            store.results = vec!["No results".to_string()];
                            for line in error.lines() {
                                store.results.push(line.to_string());
                            }
                        }
                        store.results_changed = true;
                    });
                }
                _ => {
                    self.query_textarea.input(key);
                }
            },
            Selection::Results(true) => match key.code {
                crossterm::event::KeyCode::Esc => {
                    self.selection = Selection::Results(false);
                }
                _ => {
                    self.results_textarea.input(key);
                }
            },
            Selection::Query(false) | Selection::Results(false) => match key.code {
                crossterm::event::KeyCode::Up => {
                    self.selection = Selection::Query(false);
                }
                crossterm::event::KeyCode::Down => {
                    self.selection = Selection::Results(false);
                }
                crossterm::event::KeyCode::Char('s') => {
                    app.screens
                        .push(Box::from(Settings::new(app.config.loki_url.clone())));
                }
                crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                    self.should_close = true;
                }
                crossterm::event::KeyCode::Char('d') => {
                    app.screens.push(Box::from(Remove::new(self.query_textarea.lines())));
                }
                _ => {
                    if key.code == crossterm::event::KeyCode::Enter {
                        self.selection = match self.selection {
                            Selection::Query(_) => Selection::Query(true),
                            Selection::Results(_) => Selection::Results(true),
                        }
                    }
                }
            },
        }
    }
}
