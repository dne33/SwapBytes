use crate::state::APP;
use crate::state::Screen;
use crate::network::network::Client;
use crate::ui::screens::{main_screen, login_screen, select_room_screen};
use crate::ui::screens::dm_screen::DmScreen;
use ratatui::prelude::*;

use ratatui::{
    style::{Style, Color},
    text::Span,
    layout::{Constraint, Layout},
    Frame,
    widgets::Tabs,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
};

/// Renders the tab bar with the current screen highlighted.
///
/// Displays tabs for Main, Select Room, and DM screens, highlighting the current screen.
fn render_tabs(frame: &mut Frame, area: Rect, current_screen: &Screen) {
    let tab_titles = vec![
        Span::raw("Main"),
        Span::raw("Select Room"),
        Span::raw("DM"),
    ];
    
    let current_index = match current_screen {
        Screen::MainScreen => 0,
        Screen::SelectRoomScreen => 1,
        Screen::DMScreen => 2,
        _ => 0, // Default to MainScreen if LoginScreen or undefined
    };

    let tabs = Tabs::new(tab_titles.into_iter().map(Span::from).collect::<Vec<_>>())
        .select(current_index)
        .highlight_style(Style::default().fg(Color::Yellow));

    frame.render_widget(tabs, area);
}

/// Renders the current screen content based on the application state.
///
/// Displays the appropriate screen content based on `current_screen` and updates `dm_screen` with the latest data.
pub fn render(frame: &mut Frame, dm_screen: &mut DmScreen) {
    let (current_screen, peers, usernames) = {
        let app = APP.lock().unwrap();
        (app.current_screen.clone(), app.peers.clone(), app.usernames.clone())    
    };

    // Define the layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(5),
                Constraint::Percentage(95),   // Remaining space for the screen content
            ].as_ref(),
        )
        .split(frame.area());

    if current_screen != Screen::LoginScreen {
        // Render the tabs at the top
        render_tabs(frame, chunks[0], &current_screen);
    }

    match current_screen {
        Screen::LoginScreen => login_screen::render(frame),
        Screen::MainScreen => main_screen::render(frame, chunks),
        Screen::SelectRoomScreen => select_room_screen::render(frame, chunks),
        Screen::DMScreen => dm_screen.render(frame, chunks, usernames, peers),
    }
}

/// Handles keyboard events for screen navigation and interaction.
///
/// Updates the current screen based on Tab key presses and delegates event handling to the appropriate screen module.
pub async fn handle_events(client: &mut Client, dm_screen: &mut DmScreen) -> Result<bool, std::io::Error> {
    let current_screen = {
        let app = APP.lock().unwrap();
        app.current_screen.clone()    
    };

    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            match key.code {
                // Use the Tab key to cycle through the screens
                KeyCode::Tab => {
                    let mut app = APP.lock().unwrap();
                    app.current_screen = match current_screen {
                        Screen::MainScreen => Screen::SelectRoomScreen,
                        Screen::SelectRoomScreen => Screen::DMScreen,
                        Screen::DMScreen => Screen::MainScreen,
                        _ => current_screen.clone(), // Stay on the current screen if login screen or unknown
                    };
                }
                // Optionally, handle Shift+Tab for cycling in the opposite direction
                KeyCode::BackTab => {
                    let mut app = APP.lock().unwrap();
                    app.current_screen = match current_screen {
                        Screen::MainScreen => Screen::DMScreen,
                        Screen::SelectRoomScreen => Screen::MainScreen,
                        Screen::DMScreen => Screen::SelectRoomScreen,
                        _ => current_screen.clone(), // Stay on the current screen if login screen or unknown
                    };
                }
                _ => {}
            }
            let result = match current_screen {
                Screen::LoginScreen => login_screen::handle_events(client, key).await,
                Screen::MainScreen => main_screen::handle_events(client, key).await,
                Screen::SelectRoomScreen => select_room_screen::handle_events(key).await,
                Screen::DMScreen => dm_screen.handle_events(client, key).await,
            };
            return result;
        }
    }
    Ok(false)
}
