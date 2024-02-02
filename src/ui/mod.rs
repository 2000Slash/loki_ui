use std::{
    io,
    sync::{Arc, Mutex},
};

use crossterm::event::{self, Event};
use ratatui::Frame;

use crate::{loki::Loki, LokiConfig};

pub mod screen;

pub struct Store {
    pub results: Vec<String>,
    pub results_changed: bool,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            results_changed: true,
            results: vec![
                String::from("Type a query above and press enter to see the results"),
                String::from("You can switch between query and results with ⬆️  and ⬇️."),
                String::from("Press q or esc to quit"),
            ],
        }
    }
}

pub struct App {
    pub screens: Vec<Box<dyn screen::Screen>>,
    pub loki: Loki,
    pub store: Arc<Mutex<Store>>,
    pub config: LokiConfig,
}

impl App {
    pub fn new(config: LokiConfig) -> Self {
        Self {
            screens: vec![Box::new(screen::Query::new())],
            loki: Loki::new(config.loki_url.clone()),
            store: Arc::new(Mutex::new(Store::default())),
            config,
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let length = self.screens.len();
        for i in 0..length {
            let mut screen = self.screens.remove(i);
            screen.render(frame, self);
            self.screens.insert(i, screen);
        }
    }

    pub fn handle_events(&mut self) -> io::Result<bool> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(mut key) = event::read()? {
                if key.kind != event::KeyEventKind::Press {
                    return Ok(false);
                }
                let index = self.screens.len() - 1;
                let screen = self.screens.pop();

                // Windows sets the key modifiers when using alt gr
                // We need to unset this, or these keys wont be recognized by tui-textarea
                if cfg!(windows) {
                    match key.code {
                        event::KeyCode::Char('\\') | event::KeyCode::Char('@') | event::KeyCode::Char('~') | event::KeyCode::Char('{') | event::KeyCode::Char('[') | event::KeyCode::Char(']') | event::KeyCode::Char('}')  => {
                            key.modifiers = event::KeyModifiers::empty();
                        },
                        _ => {}
                    }
                }

                if let Some(mut screen) = screen {
                    screen.handle_key_event(key, self);

                    if !screen.should_close() {
                        self.screens.insert(index, screen);
                    }

                    if self.screens.is_empty() {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
}
