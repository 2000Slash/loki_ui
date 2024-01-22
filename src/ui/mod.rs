use std::io;

use crossterm::event::{self, Event, KeyCode};
use ratatui::Frame;
pub mod screen;


pub struct App {
    screens: Vec<Box<dyn screen::Screen>>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            screens: vec![Box::new(screen::Query::new())],
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        for screen in &self.screens {
            screen.render(frame);
        }
    }

    pub fn handle_events(&mut self) -> io::Result<bool> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                for screen in &mut self.screens {
                    if screen.handle_key_event(key) {
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
}