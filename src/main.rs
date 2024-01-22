use std::io::{stdout, self};

use crossterm::{terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen}, ExecutableCommand};
use loki_ui::{ui::App, LokiConfig};
use ratatui::{Terminal, backend::CrosstermBackend};

#[tokio::main]
async fn main() -> io::Result<()> {
    color_eyre::install().unwrap();

    #[cfg(feature = "debug")]
    {
        tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
        tui_logger::set_default_level(log::LevelFilter::Trace);
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let cfg: LokiConfig = confy::load("loki_ui", None).unwrap();
    let mut app = App::new(cfg);

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| app.render(f))?;
        should_quit = app.handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
