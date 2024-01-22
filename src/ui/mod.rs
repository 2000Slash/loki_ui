use std::{io, sync::{Arc, Mutex}};

use crossterm::event::{self, Event, KeyCode};
use ratatui::Frame;

use crate::loki::Loki;
pub mod screen;

#[derive(Default)]
pub struct Store {
    pub results: Vec<crate::loki::LokiResult>
}

pub struct App {
    screens: Vec<Box<dyn screen::Screen>>,
    loki: Loki,
    store: Arc<Mutex<Store>>
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl Send for App {

}

impl App {
    pub fn new() -> Self {
        Self {
            screens: vec![Box::new(screen::Query::new())],
            loki: Loki::new(String::from("http://localhost:3100")),
            store: Arc::new(Mutex::new(Store::default()))
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        for screen in &self.screens {
            screen.render(frame, self);
        }
    }

    pub fn handle_events(&mut self) -> io::Result<bool> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                let index = self.screens.len() - 1;
                let screen = self.screens.pop();

                if let Some(mut screen) = screen {
                    let captured = screen.handle_key_event(key, &mut self.loki, self.store.clone());
                    self.screens.insert(index, screen);
                    if captured {
                        return Ok(false);
                    }
                }

                if key.kind == event::KeyEventKind::Press && ( key.code == KeyCode::Char('q') || key.code == crossterm::event::KeyCode::Esc) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub async fn run_query(&mut self, query: &str) -> Option<Vec<crate::loki::LokiResult>> {
        self.loki.query_range(query, None, None, None).await
    }
}
