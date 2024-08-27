use lazy_static::lazy_static;
use std::sync::{Mutex, Arc};
use ratatui::widgets::ListState;
use crate::ui::screens::dm_screen::DmScreen;
use libp2p::PeerId;
use std::collections::HashMap;
use crate::network::network::Client;
use crate::logger;

use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, PartialEq, Debug)]
pub enum Screen {
    LoginScreen,
    MainScreen,
    HelpScreen,
    SelectRoomScreen,
    DMScreen,
}

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    pub character_index: usize,
    /// History of recorded messages
    pub messages: Vec<String>,

    pub current_screen: Screen,

    pub username: String,

    pub connected_peers: i16,

    /// List of available rooms
    pub rooms: Vec<String>,

    /// State of the room list for selection
    pub room_state: ListState,

    pub current_room: usize,


    pub peers: Vec<PeerId>,

    pub usernames: HashMap<String, String>,

    pub peers_no_username: Vec<PeerId>,

    updating_usernames: AtomicBool, // Atomic flag to track updates
}

impl App {
    pub fn new() -> Self {
        let mut room_state = ListState::default();
        room_state.select(Some(0)); // Start with the first room selected

        Self {
            input: String::new(),
            messages: Vec::new(),
            character_index: 0,
            current_screen: Screen::LoginScreen,
            username: String::new(),
            connected_peers: 0,
            rooms: vec![
                "Global".to_string(),
                "Engineering".to_string(),
                "Sciences".to_string(),
                "Arts".to_string(),
            ],
            room_state,
            current_room: 0,
            peers: Vec::new(),
            usernames : HashMap::new(),
            peers_no_username: Vec::new(),
            updating_usernames: AtomicBool::new(false), // Initialize to false            
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&mut self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn submit_message(&mut self) {
        let final_msg = format!("{}: {}",self.username.clone(), self.input.clone() );
        self.messages.push(final_msg);
        self.input.clear();
        self.reset_cursor();
    }
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.reset_cursor();
    }

    pub async fn update_usernames(&mut self, client: &mut Client) {
        if self.updating_usernames.load(Ordering::SeqCst) {
            return; // An update is already in progress, so we skip
        }
        logger::info!("{:?}, {:?}, {:?}", self.usernames.len(), self.peers.len(), self.peers_no_username.len());
        self.updating_usernames.store(true, Ordering::SeqCst);
        

        if self.usernames.len() > (self.peers.len() - self.peers_no_username.len()) {
            let mut new_usernames = HashMap::new();
            for peer in &self.peers {
                let peer_to_string = peer.to_string();
                if let Some(username) = self.usernames.get(&peer_to_string) {
                    new_usernames.insert(peer_to_string, username.clone());
                }
            }
            self.usernames = new_usernames;

        } else if self.usernames.len() < (self.peers.len() - self.peers_no_username.len()) {
            for peer in &self.peers {
                let peer_to_string = peer.to_string();
                if !self.usernames.contains_key(&peer_to_string) {
                    client.get_username(peer_to_string).await;
                }
            }
        } else if self.peers_no_username.len() > 0 {
            for peer in &self.peers {
                let peer_to_string = peer.to_string();
                if !self.usernames.contains_key(&peer_to_string) {
                    client.get_username(peer_to_string).await;
                }
            }
        }
        self.updating_usernames.store(false, Ordering::SeqCst);
        logger::info!("{:?}", self.usernames);
    }

}

lazy_static! {
    pub static ref APP: Arc<Mutex<App>> = Arc::new(Mutex::new(App::new()));
}