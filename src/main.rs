use std::{error::Error, io};

use ratatui::{
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
};

use tokio::task::spawn;

mod state;
use state::APP;
use state::Screen::DMScreen;

pub mod logger;

pub mod ui {
    pub mod screens {
        pub mod main_screen;
        pub mod help_screen;
        pub mod login_screen;
        pub mod select_room_screen;
        pub mod dm_screen;
    }
    pub mod ui_router;
}
pub mod network {
    pub mod network_behaviour {
        pub mod mdns_behaviour;
        pub mod gossipsub_behaviour;
        pub mod kademlia_behaviour;
        pub mod request_response_behaviour;
    }
    pub mod network;
}

use ui::screens::dm_screen::DmScreen;
use ui::ui_router::render;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let (mut network_client, network_event_loop) =
        network::network::new().await?;

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
    let mut dm_screen = DmScreen::new();
    // Log in sequence
    let mut break_loop = false;
    while !break_loop {
        terminal.draw(|f| render(f, &mut dm_screen))?;
        break_loop = ui::ui_router::handle_events(&mut network_client, &mut dm_screen).await?;
    }
    logger::info!("User has added their username");

    APP.lock().unwrap().current_screen = DMScreen;
    
    // create app and run it
    break_loop = false;
    while !break_loop {
        let mut app = APP.lock().unwrap();
        if app.current_screen == DMScreen && app.peers.len() >= 1 {
            // instead of handing client into render funtion send update instructions here before the render
            app.update_usernames(&mut network_client).await;
        }
        drop(app);

        terminal.draw(|f| render(f, &mut dm_screen))?;
        break_loop = ui::ui_router::handle_events(&mut network_client, &mut dm_screen).await?;
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


