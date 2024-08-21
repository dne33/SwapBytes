use std::{error::Error, io};

use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
    widgets::{Block, List, ListItem, Paragraph},
};

mod network;

use tokio::task::spawn;
use futures::prelude::*;
use futures::StreamExt;
use std::path::PathBuf;

mod state;
use state::APP;
use state::Screen;

pub mod logger;

mod UI;
use UI::ui::render_ui;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let (mut network_client, mut network_events, network_event_loop) =
        network::new().await?;

    // Spawn the network task for it to run in the background.
    spawn(network_event_loop.run());

    // In case a listen address was provided use it, otherwise listen on any
    // address.
    network_client
            .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
            .await
            .expect("Listening not to fail.");
    
    logger::initialize();
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    loop {
        terminal.draw(|f| render_ui(f))?;
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let mut app = APP.lock().unwrap();
                match key.code {
                    KeyCode::Enter => {
                        logger::info!("Shoulder tapping");
                        network_client.submit_message(app.input.clone()).await;
                        app.submit_message();
                    }
                    KeyCode::Char(to_insert) => {
                        app.enter_char(to_insert);
                    }
                    KeyCode::Backspace => {
                        app.delete_char();
                    }
                    KeyCode::Left => {
                        app.move_cursor_left();
                    }
                    KeyCode::Right => {
                        app.move_cursor_right();
                    },
                    KeyCode::Tab => {
                    },
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;


    Ok(())
}
