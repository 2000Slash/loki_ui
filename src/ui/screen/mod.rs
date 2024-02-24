use crossterm::event::KeyEvent;
use ratatui::Frame;

mod alert;
mod query;
mod remove;
mod settings;

pub use alert::Alert;
pub use query::Query;

use super::App;

pub trait Screen {
    fn should_close(&self) -> bool;
    fn render(&mut self, frame: &mut Frame, app: &App);
    fn handle_key_event(&mut self, key: KeyEvent, app: &mut App);
}
