use crate::state::APP;
use crate::state::Screen;
use crate::network::network::Client;
use crate::ui::screens::{main_screen, help_screen, login_screen, select_room_screen};
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
    widgets::{Block, List, ListItem, Paragraph, Borders},
};

pub fn render(frame: &mut Frame) {
    let current_screen = APP.lock().unwrap().current_screen.clone();
    match current_screen {
        Screen::LoginScreen => login_screen::render(frame),
        Screen::MainScreen => main_screen::render(frame),
        Screen::HelpScreen => help_screen::render(frame),
        Screen::SelectRoomScreen => select_room_screen::render(frame),
        Screen::SelectDMScreen => select_dm_screen::render(frame),
    }
}

pub async fn handle_events(client: &mut Client) -> Result<bool, std::io::Error> {
    let current_screen = APP.lock().unwrap().current_screen.clone();
    match current_screen {
        Screen::LoginScreen => login_screen::handle_events(client).await,
        Screen::MainScreen => main_screen::handle_events(client).await,
        Screen::HelpScreen => help_screen::handle_events().await,
        Screen::SelectRoomScreen => select_room_screen::handle_events().await,
        Screen::SelectDMScreen => select_room_screen::handle_events().await,
    }
}