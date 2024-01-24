use std::{io, sync::{Arc, Mutex}};

use crossterm::event::{self, Event};
use ratatui::Frame;

use crate::{loki::Loki, LokiConfig};

pub mod screen;


pub struct Store {
    pub results: Vec<String>
}

impl Default for Store {
    fn default() -> Self {
        Self {
            results: vec![String::from("Type a query above and press enter to see the results"), String::from("You can switch between query and results with ⬆️  and ⬇️."), String::from("Press q or esc to quit")]
        }
    }
}

pub struct App {
    screens: Vec<Box<dyn screen::Screen>>,
    loki: Loki,
    store: Arc<Mutex<Store>>
}

impl App {
    pub fn new(config: LokiConfig) -> Self {
        Self {
            screens: vec![Box::new(screen::Query::new())],
            loki: Loki::new(config.loki_url),
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
                if key.kind != event::KeyEventKind::Press {
                    return Ok(false);
                }
                let index = self.screens.len() - 1;
                let screen = self.screens.pop();

                if let Some(mut screen) = screen {
                    screen.handle_key_event(key, &mut self.loki, self.store.clone(), &mut self.screens);
                    
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
