use crossterm::event::KeyEvent;
use ratatui::Frame;

mod query;
mod settings;

pub use query::Query;

use super::App;

pub trait Screen {
    fn should_close(&self) -> bool;
    fn render(&self, frame: &mut Frame, app: &App);
    fn handle_key_event(&mut self, key: KeyEvent, app: &mut App);
}
