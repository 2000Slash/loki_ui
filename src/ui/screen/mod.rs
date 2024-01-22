use crossterm::event::KeyEvent;
use ratatui::Frame;

mod query;

pub use query::Query;

pub(crate) trait Screen {
    fn render(&self, frame: &mut Frame);
    fn handle_key_event(&mut self, key: KeyEvent) -> bool;
}