use std::sync::{Arc, Mutex};

use crossterm::event::KeyEvent;
use ratatui::Frame;

mod query;
mod settings;

pub use query::Query;

use crate::loki::Loki;

use super::{App, Store};

pub(crate) trait Screen {
    fn should_close(&self) -> bool;
    fn render(&self, frame: &mut Frame, app: &App);
    fn handle_key_event(&mut self, key: KeyEvent, loki: &mut Loki, store: Arc<Mutex<Store>>, screens: &mut Vec<Box<dyn Screen>>);
}
