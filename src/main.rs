use std::io::{stdout, self};

use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, ExecutableCommand};
use loki_ui::ui::App;
use ratatui::{Terminal, backend::CrosstermBackend};


#[tokio::main]
async fn main() -> io::Result<()> {
    color_eyre::install().unwrap();
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| app.render(f))?;
        should_quit = app.handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}