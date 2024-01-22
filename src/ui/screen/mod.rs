use std::sync::{Arc, Mutex};

use crossterm::event::KeyEvent;
use ratatui::Frame;

mod query;

pub use query::Query;

use crate::loki::Loki;

use super::{App, Store};

pub(crate) trait Screen {
    fn render(&self, frame: &mut Frame, app: &App);
    fn handle_key_event(&mut self, key: KeyEvent, loki: &mut Loki, store: Arc<Mutex<Store>>) -> bool;
}
