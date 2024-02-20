use std::io::{self, stdout};

use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use loki_ui::{ui::App, LokiConfig};
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() -> io::Result<()> {
    #[cfg(feature = "debug")]
    {
        color_eyre::install().unwrap();
        tui_logger::init_logger(log::LevelFilter::Trace).unwrap();
        tui_logger::set_default_level(log::LevelFilter::Trace);
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let envs = envy::from_env::<LokiConfig>();
    let cfg = match envs {
        Ok(envs) => envs,
        Err(_) => confy::load("loki_ui", None).unwrap(),
    };
    let mut app = App::new(cfg);

    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| app.render(f))?;
        should_quit = app.handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    confy::store("loki_ui", None, &app.config).unwrap();
    Ok(())
}
