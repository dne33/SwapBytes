use crate::state::APP;
use crate::state::Screen;
use crate::network::network::Client;
use crate::ui::screens::{main_screen, help_screen, login_screen, select_room_screen};
use crate::ui::screens::dm_screen::DmScreen;
use ratatui::prelude::*;
use crate::logger;

pub fn render(frame: &mut Frame, dm_screen: &mut DmScreen) {
    let (current_screen, peers, usernames) = {
        let app = APP.lock().unwrap();
        (app.current_screen.clone(), app.peers.clone(), app.usernames.clone())    
    };
    match current_screen {
        Screen::LoginScreen => login_screen::render(frame),
        Screen::MainScreen => main_screen::render(frame),
        Screen::HelpScreen => help_screen::render(frame),
        Screen::SelectRoomScreen => select_room_screen::render(frame),
        Screen::DMScreen => dm_screen.render(frame, usernames, peers),
    }
}

pub async fn handle_events(client: &mut Client, dm_screen: &mut DmScreen) -> Result<bool, std::io::Error> {
    let current_screen = {
        let app = APP.lock().unwrap();
        app.current_screen.clone()    
    };
    match current_screen {
        Screen::LoginScreen => login_screen::handle_events(client).await,
        Screen::MainScreen => main_screen::handle_events(client).await,
        Screen::HelpScreen => help_screen::handle_events().await,
        Screen::SelectRoomScreen => select_room_screen::handle_events().await,
        Screen::DMScreen => dm_screen.handle_events().await,
    }
}