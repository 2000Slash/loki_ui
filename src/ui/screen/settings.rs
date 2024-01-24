use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use tui_textarea::TextArea;

use super::Screen;



enum Selection {
    Url
}


pub struct Settings<'a> {
    loki_url: TextArea<'a>,
    selection: Selection,
    should_close: bool
}

impl Default for Settings<'_> {
    fn default() -> Self {
        Self { loki_url: Default::default(), selection: Selection::Url, should_close: false }
    }
}

impl Screen for Settings<'_> {
    fn should_close(&self) -> bool {
        self.should_close
    }

    fn render(&self, frame: &mut ratatui::prelude::Frame, _app: &crate::ui::App) {
        let block = Block::default().title("Settings").borders(Borders::ALL);
        let mut size = frame.size();
        size.width -= 8;
        size.height -= 10;
        size.x += 4;
        size.y += 5;
        frame.render_widget(Clear, size);
        size.width -= 2;
        size.x += 1;
        frame.render_widget(Paragraph::new("Hello world").block(block), size);
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent, _loki: &mut crate::loki::Loki, _store: std::sync::Arc<std::sync::Mutex<crate::ui::Store>>, _screens: &mut Vec<Box<dyn Screen>>) {
        match key.code {
            crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Char('q') => {
                self.should_close = true;
            },
            _ => { }
        }
    }
}