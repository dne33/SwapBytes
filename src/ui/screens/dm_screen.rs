use ratatui::{
    style::{Modifier, Style, Color},
    text::{Line, Span},
    layout::{Constraint, Layout, Position},
    Frame,
    widgets::{List, ListItem, Paragraph, ListState, Block, Borders},
    crossterm::event::{KeyCode, KeyEvent},
};
use crate::network::network::Client;
use crate::logger;
use crate::APP;
use std::collections::HashMap;
use libp2p::{gossipsub, PeerId};
use std::rc::Rc;

/// Represents the Direct Message (DM) screen state.
pub struct DmScreen {
    pub private_messages: HashMap<String, Vec<String>>,
    pub people_state: ListState,
    pub request_state: ListState,
    pub selected_person: usize,
    pub in_sidebar: bool,
    pub in_requests: bool,
    pub usernames: HashMap<String, String>,
    pub peers: Vec<PeerId>,
}

impl DmScreen {
    /// Creates a new `DmScreen` with default state.
    pub fn new() -> Self {
        let mut people_state = ListState::default();
        people_state.select(Some(0));
        let mut request_state = ListState::default();
        request_state.select(Some(0));
        Self {
            private_messages: HashMap::new(),
            people_state,
            request_state,
            selected_person: 0,
            in_sidebar: false,
            in_requests: false,
            usernames: HashMap::new(),
            peers: Vec::new(),
        }
    }

    /// Renders the DM screen including input area, messages, people list, and incoming requests.
    ///
    /// Displays the input field, messages with the selected peer, a list of people, and incoming requests.
    pub fn render(&mut self, frame: &mut Frame, chunk: Rc<[ratatui::layout::Rect]>, usernames: HashMap<String, String>, peers: Vec<PeerId>) {
        let horizontal = Layout::horizontal([
            Constraint::Length(30),
            Constraint::Min(1),
        ]);
        let [sidebar_area, main_area] = horizontal.areas(chunk[1]);

        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [input_area, messages_area] = vertical.areas(main_area);
        
        let vertical_sidebar = Layout::vertical([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ]);

        let [peer_area, request_area] = vertical_sidebar.areas(sidebar_area);

        // Input area
        let input_style = if self.in_sidebar || self.in_requests {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let app = APP.lock().unwrap();
        let input = Paragraph::new(app.input.as_str())
            .style(input_style)
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);

        if !self.in_sidebar && !self.in_requests {
            frame.set_cursor_position(Position {
                x: input_area.x + app.character_index as u16 + 1,
                y: input_area.y + 1,
            });
        }

        // Retrieve current user's peer ID
        let current_user_peer_id = app.my_peer_id.as_ref().map(|id| id.to_string()).unwrap_or_default();
        
        // Retrieve the selected peer's ID and username
        let selected_peer_id = peers.get(self.selected_person)
            .map(|peer_id| peer_id.to_string())
            .unwrap_or_default();
        let selected_username = usernames.get(&selected_peer_id)
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string());

        // Construct the message key using the sorted peer IDs
        let mut peer_ids = vec![current_user_peer_id.clone(), selected_peer_id.clone()];
        peer_ids.sort(); // Sort alphabetically
        let message_key = peer_ids.join("_");

        // Messages area with the username of the selected peer as the title
        let binding = vec!["Ensure a peer is connected to DM".to_string()];
        let private_messages = app.private_messages.get(&message_key)
            .unwrap_or(&binding)
            .iter()
            .map(|m| ListItem::new(Line::from(Span::raw(m))))
            .collect::<Vec<_>>();
        let message_store = List::new(private_messages)
            .block(Block::bordered().title(format!("Messages with {}", selected_username)));
        frame.render_widget(message_store, messages_area);
        self.peers = peers.clone();
        let peer_list: Vec<String> = peers.iter().map(|peer_id| format!("{}", peer_id.to_string())).collect();  
        let peer_items: Vec<ListItem> = peer_list
        .iter()
        .filter_map(|peer| {
            usernames.get(peer).map(|username| ListItem::new(format!("{}", username)))
        })
        .collect();

        // Sidebar (people list)
        let people_style = if !self.in_sidebar {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let people_list = List::new(peer_items)
            .block(Block::default().borders(Borders::ALL).title("People"))
            .style(people_style)
            .highlight_style(if self.in_sidebar {
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)
            } else {
                Style::default().add_modifier(Modifier::REVERSED)
            })
            .highlight_symbol(">>");
        frame.render_stateful_widget(people_list, peer_area, &mut self.people_state);    

        let current_requests = &app.current_requests;
        self.usernames = usernames.clone();
        let request_items: Vec<String> = current_requests.iter().map(|request| {
            // Safely retrieve the username using the peer_id
            match self.usernames.get(&request.peer_id.to_string()) {
                Some(username) => format!("{} - {}", username, request.request_string),
                None => {
                    logger::error!("Username for peer_id {} not found", request.peer_id);
                    format!("Unknown user - {}", request.request_string)
                }
            }
        }).collect();
        let request_style = if !self.in_requests {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Yellow)
        };
        let requests = List::new(request_items.into_iter().map(ListItem::new).collect::<Vec<_>>())
            .block(Block::default().borders(Borders::ALL).title("Incoming Requests"))
            .style(request_style)
            .highlight_style(if self.in_requests {
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)
            } else {
                Style::default().add_modifier(Modifier::REVERSED)
            })
            .highlight_symbol(">>");

        frame.render_stateful_widget(requests, request_area, &mut self.request_state);
    }
    /// Handles keyboard events for the DM screen.
    ///
    /// Processes key inputs for navigating lists, sending messages or requests, and toggling UI modes.
    /// Returns `Ok(true)` if the Escape key is pressed to exit the application, otherwise `Ok(false)`.
    pub async fn handle_events(&mut self, client: &mut Client, key: KeyEvent) -> Result<bool, std::io::Error> {
        match key.code {
            KeyCode::Enter => self.handle_enter(client).await,
            KeyCode::Char('~') => {
                self.toggle_ui_modes();
                
            }
            KeyCode::Char(to_insert) => {
                self.handle_char(to_insert);
            }
            KeyCode::Backspace => {
                self.handle_backspace();
            }
            KeyCode::Left => {
                self.handle_left();
            }
            KeyCode::Right => {
                self.handle_right();
            }
            KeyCode::Up => {
                self.handle_up();
            }
            KeyCode::Down => {
                self.handle_down();
            }
            KeyCode::Esc => return Ok(true),
            e => {
                logger::error!("{:?}", e);
            }
        }
        Ok(false)

    }

    // Handles the enter key press, the resulting action depends which component the user is currently in.
    async fn handle_enter(&mut self, client: &mut Client) {
        logger::info!("Enter pressed");

        if self.in_sidebar {
            if let Some(peer_id) = self.select_person() {
                logger::info!("Selected Peer ID: {:?}", peer_id);
            }
            self.in_sidebar = !self.in_sidebar;
        } else if self.in_requests {
            let mut app = APP.lock().unwrap();
            logger::info!("Sending File Response");
            logger::info!("{:?}", self.request_state.clone());

            let index = self.request_state.clone().selected()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "No request selected"));
            let request = app.current_requests.remove(index.expect("Invalid Index"));
            let channel = request.response_channel;
            let input = request.request_string.clone();

            client.send_response(input.clone(), input.clone(), channel).await;
            app.input.clear();
            app.character_index = 0;
        } else {
            let mut app = APP.lock().unwrap();
            let input = app.input.clone();

            if !input.is_empty() && !input.starts_with("!request file") {
                let my_peer_id = app.my_peer_id.as_ref().map_or("No Peer ID".to_string(), |peer_id| peer_id.to_string());
                logger::info!("peers: {:?}, selected: {:?}", self.peers.clone(), self.selected_person.clone());
                let peer_id = self.peers[self.selected_person].clone().to_string();
                let mut peer_ids = vec![my_peer_id, peer_id];
                peer_ids.sort();
                let topic = gossipsub::IdentTopic::new(peer_ids.join("_"));

                client.submit_message(input.clone(), topic).await;

                app.submit_private_message(peer_ids.join("_"));
                app.input.clear();
                app.character_index = 0;
            } else if input.starts_with("!request file") {
                logger::info!("Sending File Request");
                let file: Vec<_> = input.split_whitespace().collect();
                let peer_id = self.peers[self.selected_person].clone();
                client.send_request(file.get(file.len()-1).expect("").to_string(), peer_id).await;
                app.input.clear();
                app.character_index = 0;
            }
        }
    }

    /// Toggles between sidebar and request modes.
    fn toggle_ui_modes(&mut self) {
        if self.in_sidebar {
            self.in_sidebar = false;
            self.in_requests = true;
        } else if self.in_requests {
            self.in_requests = false;
        } else {
            self.in_sidebar = true;
        }
    }

    /// Handles character input, inserting it into the application state if not in sidebar mode.
    fn handle_char(&mut self, to_insert: char) {
        logger::info!("Pressed a Char");
        if !self.in_sidebar {
            let mut app = APP.lock().unwrap();
            app.enter_char(to_insert);
        }
    }

    /// Handles backspace input, deleting a character from the application state if not in sidebar mode.
    fn handle_backspace(&mut self) {
        if !self.in_sidebar {
            let mut app = APP.lock().unwrap();
            app.delete_char();
        }
    }

    /// Handles left arrow input, moving the cursor left in the application state if not in sidebar mode.
    fn handle_left(&mut self) {
        if !self.in_sidebar {
            let mut app = APP.lock().unwrap();
            app.move_cursor_left();
        }
    }

    /// Handles right arrow input, moving the cursor right in the application state if not in sidebar mode.
    fn handle_right(&mut self) {
        if !self.in_sidebar {
            let mut app = APP.lock().unwrap();
            app.move_cursor_right();
        }
    }

    /// Handles up arrow input, navigating up in the list of users or requests, depending on the current mode.
    fn handle_up(&mut self) {
        if self.in_sidebar {
            let user_count = self.usernames.len();
            if user_count > 0 {
                let i = match self.people_state.selected() {
                    Some(0) => user_count - 1,
                    Some(i) => i - 1,
                    None => 0,
                };
                self.people_state.select(Some(i));
            }
        } else if self.in_requests {
            let request_count = APP.lock().unwrap().current_requests.len();
            if request_count > 0 {
                let i = match self.request_state.selected() {
                    Some(0) => request_count - 1,
                    Some(i) => i - 1,
                    None => 0,
                };
                self.request_state.select(Some(i));
            }
        }
    }

    /// Handles down arrow input, navigating down in the list of users or requests, depending on the current mode.
    fn handle_down(&mut self) {
        if self.in_sidebar {
            let user_count = self.usernames.len();
            if user_count > 0 {
                let i = match self.people_state.selected() {
                    Some(i) if i >= user_count - 1 => 0,
                    Some(i) => i + 1,
                    None => 0,
                };
                self.people_state.select(Some(i));
            }
        } else if self.in_requests {
            let request_count = APP.lock().unwrap().current_requests.len();
            if request_count > 0 {
                let i = match self.request_state.selected() {
                    Some(i) if i >= request_count - 1 => 0,
                    Some(i) => i + 1,
                    None => 0,
                };
                self.request_state.select(Some(i));
            }
        }
    }


    /// Selects a person from the list based on the current selection state.
    ///
    /// Updates the `selected_person` index if a valid selection is made.
    fn select_person(&mut self) -> Option<String> {
        if let Some(selected) = self.people_state.selected() {
            if selected < self.peers.len() {
                let peer_id = self.peers[selected].to_string();
                logger::info!("Selected peer ID: {:?}", peer_id);
                self.selected_person = selected;
            }
        }
        None
    }
}
