use lazy_static::lazy_static;
use std::sync::{Mutex, Arc};
use ratatui::widgets::ListState;
use libp2p::PeerId;
use std::collections::HashMap;
use crate::network::network::{Response, Client};
use libp2p_request_response::ResponseChannel;
use crate::logger;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, PartialEq, Debug)]
pub enum Screen {
    LoginScreen,
    MainScreen,
    SelectRoomScreen,
    DMScreen,
}

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area
    pub character_index: usize,
    /// History of recorded messages for public rooms
    pub public_messages: HashMap<String, Vec<String>>,
    /// History of recorded messages for private conversations
    pub private_messages: HashMap<String, Vec<String>>,
    /// Currently displayed screen
    pub current_screen: Screen,
    /// Username of the current user
    pub username: String,
    /// Number of connected peers
    pub connected_peers: i16,
    /// List of available rooms
    pub rooms: Vec<String>,
    /// State for managing the room list selection
    pub room_state: ListState,
    /// Index of the currently selected room
    pub current_room: usize,
    /// List of peer IDs
    pub peers: Vec<PeerId>,
    /// Mapping of peer IDs to usernames
    pub usernames: HashMap<String, String>,
    /// List of peers without usernames
    pub peers_no_username: Vec<PeerId>,
    /// Flag indicating if usernames are being updated
    updating_usernames: AtomicBool,
    /// ID of the current peer
    pub my_peer_id: Option<PeerId>,
    /// List of current requests
    pub current_requests: Vec<RequestItem>,
}

impl App {
    // Creates a new App instance with default values
    pub fn new() -> Self {
        let mut room_state = ListState::default();
        room_state.select(Some(0)); // Start with the first room selected

        let rooms = vec![
            "global".to_string(),
            "engineering".to_string(),
            "sciences".to_string(),
            "arts".to_string(),
        ];

        // Initialize a HashMap to store messages for each room
        let mut public_messages = HashMap::new();
        for room in &rooms {
            public_messages.insert(room.clone(), Vec::new());
        }

        Self {
            input: String::new(),
            public_messages,
            private_messages: HashMap::new(),
            character_index: 0,
            current_screen: Screen::LoginScreen,
            username: String::new(),
            connected_peers: 0,
            rooms,
            room_state,
            current_room: 0,
            peers: Vec::new(),
            usernames: HashMap::new(),
            peers_no_username: Vec::new(),
            updating_usernames: AtomicBool::new(false), // Initialize to false
            my_peer_id: None,
            current_requests: Vec::new(),
        }
    }

    // Moves the cursor one position to the left
    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    // Moves the cursor one position to the right
    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    // Inserts a new character at the cursor position
    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    // Returns the byte index based on the character position
    fn byte_index(&mut self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    // Deletes the character before the cursor
    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before and after the selected character
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Reconstruct the string excluding the selected character
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    // Clamps the cursor position within the valid range
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    // Resets the cursor position to the start
    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    // Submits a public message to the current room
    pub fn submit_public_room_message(&mut self) {
        let final_msg = format!("{}: {}", self.username.clone(), self.input.clone());
        // Determine the current room
        if let Some(current_room_name) = self.rooms.get(self.current_room) {
            // Push the message to the appropriate room's message vector
            self.public_messages.entry(current_room_name.clone())
                .or_insert_with(Vec::new)
                .push(final_msg.clone());
        }

        self.input.clear();
        self.reset_cursor();
    }

    // Submits a private message to a specific topic
    pub fn submit_private_message(&mut self, topic: String) {
        let final_msg = format!("{}: {}", self.username.clone(), self.input.clone());
        // Push the message to the appropriate topic's message vector
        self.private_messages.entry(topic.clone())
            .or_insert_with(Vec::new)
            .push(final_msg.clone());
        self.input.clear();
        self.reset_cursor();
    }

    // Clears the input field
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.reset_cursor();
    }

    // Updates the list of usernames based on connected peers
    pub async fn update_usernames(&mut self, client: &mut Client) {
        if self.updating_usernames.load(Ordering::SeqCst) {
            return; // An update is already in progress
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

pub struct RequestItem {
    pub peer_id: PeerId,
    pub request_string: String,
    pub response_channel: ResponseChannel<Response>,
}

lazy_static! {
    // Global application state
    pub static ref APP: Arc<Mutex<App>> = Arc::new(Mutex::new(App::new()));
}
