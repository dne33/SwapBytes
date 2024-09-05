use crate::state::APP;
use crate::state::Screen;
use crate::network::network::Client;
use crate::ui::screens::{main_screen, help_screen, login_screen, select_room_screen};
use crate::ui::screens::dm_screen::DmScreen;
use ratatui::prelude::*;
use crate::logger;
use ratatui::{
    style::{Modifier, Style, Color},
    text::{Line, Span, Text},
    layout::{Constraint, Layout, Position},
    Frame,
    prelude::*,
    widgets::{List, ListItem, Paragraph, ListState, Block, Borders, Tabs},
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyEvent},
};

fn render_tabs(frame: &mut Frame, area: Rect, current_screen: &Screen) {
    let tab_titles = vec![
        Span::raw("Main"),
        Span::raw("Help"),
        Span::raw("Select Room"),
        Span::raw("DM"),
    ];
    
    let current_index = match current_screen {
        Screen::MainScreen => 0,
        Screen::HelpScreen => 1,
        Screen::SelectRoomScreen => 2,
        Screen::DMScreen => 3,
        _ => 0, // Default to MainScreen if LoginScreen or undefined
    };

    let tabs = Tabs::new(tab_titles.into_iter().map(Span::from).collect::<Vec<_>>())
        .select(current_index)
        .highlight_style(Style::default().fg(Color::Yellow));

    frame.render_widget(tabs, area);
}

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
        .split(frame.size());

    if current_screen != Screen::LoginScreen {
        // Render the tabs at the top
        render_tabs(frame, chunks[0], &current_screen);
    }

    match current_screen {
        Screen::LoginScreen => login_screen::render(frame),
        Screen::MainScreen => main_screen::render(frame, chunks),
        Screen::HelpScreen => help_screen::render(frame, chunks),
        Screen::SelectRoomScreen => select_room_screen::render(frame, chunks),
        Screen::DMScreen => dm_screen.render(frame, chunks, usernames, peers),
    }
}

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
                        Screen::MainScreen => Screen::HelpScreen,
                        Screen::HelpScreen => Screen::SelectRoomScreen,
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
                        Screen::HelpScreen => Screen::MainScreen,
                        Screen::SelectRoomScreen => Screen::HelpScreen,
                        Screen::DMScreen => Screen::SelectRoomScreen,
                        _ => current_screen.clone(), // Stay on the current screen if login screen or unknown
                    };
                }
                _ => {}
            }
            let result = match current_screen {
                Screen::LoginScreen => login_screen::handle_events(client, key).await,
                Screen::MainScreen => main_screen::handle_events(client, key).await,
                Screen::HelpScreen => help_screen::handle_events(key).await,
                Screen::SelectRoomScreen => select_room_screen::handle_events(key).await,
                Screen::DMScreen => dm_screen.handle_events(client, key).await,
            };
            return result
        }
    }
    Ok(false)
}