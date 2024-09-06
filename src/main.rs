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
use state::Screen;

pub mod logger;

pub mod ui {
    pub mod screens {
        pub mod main_screen;
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
use network::network::Client;

/**
 * Sets up the terminal by enabling raw mode, switching to the alternate screen,
 * and enabling mouse capture.
 */
fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/**
 * Restores the terminal state by disabling raw mode, switching back to the
 * normal screen, and disabling mouse capture.
 */
fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/**
 * Initializes the network by creating a client and starting the event loop in a background task.
 */
async fn init_network() -> Result<Client, Box<dyn Error>> {
    let (mut network_client, network_event_loop) = network::network::new().await?;
    spawn(network_event_loop.run());
    network_client
        .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
        .await
        .expect("Listening not to fail.");
    Ok(network_client)
}

/**
 * Handles the main event loop for the DM screen, drawing the UI and processing events.
 */
async fn run_login_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    network_client: &mut Client,
    dm_screen: &mut DmScreen,
) -> Result<(), Box<dyn Error>> {
    let mut break_loop = false;

    while !break_loop {
        terminal.draw(|f| render(f, dm_screen))?;
        break_loop = ui::ui_router::handle_events(network_client, dm_screen).await?;
    }

    Ok(())
}

/**
 * Handles the screen transitions and updating logic based on the current screen.
 */
async fn run_screen_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    network_client: &mut Client,
    dm_screen: &mut DmScreen,
) -> Result<(), Box<dyn Error>> {
    let mut break_loop = false;

    while !break_loop {
        let mut app = APP.lock().unwrap();
        logger::info!("Curr Screen: {:?}", app.current_room.clone());
        match app.current_screen.clone() {
            Screen::DMScreen if app.peers.len() >= 1 => {
                app.update_usernames(network_client).await;
            }
            Screen::SelectRoomScreen => {
                network_client.get_rooms().await;
            }
            _ => {}
        }
        drop(app);

        // Draw the current UI and process events
        terminal.draw(|f| render(f, dm_screen))?;
        break_loop = ui::ui_router::handle_events(network_client, dm_screen).await?;
    }

    Ok(())
}

/**
 * Main entry point for the application.
 * Initializes the network, sets up the terminal, and runs the main event loop.
 */
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger and network
    logger::initialize();
    let mut network_client = init_network().await?;

    // Setup terminal
    let mut terminal = setup_terminal()?;
    let mut dm_screen = DmScreen::new();

    // Log in sequence
    run_login_loop(&mut terminal, &mut network_client, &mut dm_screen).await?;
    logger::info!("User has added their username");
    
    APP.lock().unwrap().current_screen = Screen::MainScreen;

    // Run the event loop for the login screen and main application
    run_screen_loop(&mut terminal, &mut network_client, &mut dm_screen).await?;

    // Restore terminal state after program exits
    restore_terminal(&mut terminal)?;

    Ok(())
}
