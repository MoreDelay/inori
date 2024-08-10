extern crate mpd;

use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        terminal::{
            disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
            LeaveAlternateScreen,
        },
        ExecutableCommand,
    },
    Terminal,
};

use std::io::stdout;
mod event_handler;
mod model;
mod update;
mod util;
mod view;

use event_handler::{Event, Result};

fn main() -> Result<()> {
    // stdout().execute(EnterAlternateScreen)?;
    // enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    // terminal.clear()?;

    let mut model = model::Model::new().expect("Failed to init.");
    println!("{:?}",
             model.conn.list_group_2(("albumartistsort".into(), "albumartist".into()))?);

    panic!("test");

    let event_handler = event_handler::EventHandler::new();
    while model.state != model::State::Done {
        terminal.draw(|f| view::view(&model, f))?;
        match event_handler.next()? {
            Event::Tick => update::update_tick(&mut model)?,
            Event::Key(k) => update::handle_event(&mut model, k)?,
        }
    }

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
