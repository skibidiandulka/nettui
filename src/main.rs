use anyhow::Result;
use nettui::{
    app::{App, AppConfig},
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

#[tokio::main]
async fn main() -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;

    let mut tui = Tui::new(terminal);
    tui.init()?;

    let config = AppConfig::default();
    let mut events = EventHandler::new(config.tick_ms);
    let mut app = App::new(config).await?;

    while app.running {
        tui.draw(&mut app)?;

        match events.next().await? {
            Event::Tick => app.tick().await?,
            Event::Key(key_event) => {
                handle_key_events(key_event, &mut app).await?;
            }
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;
    Ok(())
}
