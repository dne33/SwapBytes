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

pub struct DmScreen {
    pub private_messages: HashMap <String, Vec<String>>,
    pub people_state: ListState,
    pub request_state: ListState,

    pub selected_person: usize,
    pub in_sidebar: bool,
    pub in_requests: bool,
    pub usernames: HashMap<String, String>,
    pub peers: Vec<PeerId>,
}

impl DmScreen {
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

    pub fn render(&mut self, frame: &mut Frame,  chunk: Rc<[ratatui::layout::Rect]>, usernames: HashMap<String, String>, peers: Vec<PeerId>) {
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
        let current_user_peer_id = {
            app.my_peer_id.as_ref().map(|id| id.to_string()).unwrap_or_default()
        };
        
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

        // Messages area with the username of the selected peer as the title\
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
            // .style()
            .highlight_style(if self.in_sidebar {
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)
            } else {
                Style::default().add_modifier(Modifier::REVERSED)
            })
            .highlight_symbol(">>");
        frame.render_stateful_widget(people_list, peer_area, &mut self.people_state);    

        let current_requests = &app.current_requests;
        self.usernames = usernames.clone();
        let request_items: Vec<String> = current_requests.iter().map(|request| format!("{} - {}", self.usernames.get(&request.peer_id.to_string()).expect(""), request.request_string)).collect();  
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

    pub async fn handle_events(&mut self, client: &mut Client, key: KeyEvent) -> Result<bool, std::io::Error> {
        match key.code {
            KeyCode::Enter => {
                logger::info!("Enter pressed");

                if self.in_sidebar {
                    if let Some(peer_id) = self.select_person() {
                        logger::info!("Selected Peer ID: {:?}", peer_id);
                        // Use the peer_id as needed here
                    }
                    self.in_sidebar = !self.in_sidebar;
                } else if self.in_requests {
                    let mut app = APP.lock().unwrap();
                    logger::info!("Sending File Response");
                    logger::info!("{:?}", self.request_state.clone());

                    let index = self.request_state.clone().selected().expect("");
                    let request = app.current_requests.remove(index);
                    let channel = request.response_channel; // Move the channel out
                    let input = request.request_string;
                    
                    client.send_response(input.clone(), input.clone(), channel).await;
                    app.input.clear();
                    app.character_index = 0;
                } else {
                    let input = APP.lock().unwrap().input.clone();
                    
                    if input.len() != 0 && !input.starts_with("!request file") {
                        let app = APP.lock().unwrap();
                        let my_peer_id = match &app.my_peer_id {
                            Some(peer_id) => peer_id.to_string(),
                            None => "No Peer ID".to_string(), // Provide a default or placeholder
                        };
                        drop(app);
                        logger::info!("peers: {:?}, selected: {:?}", self.peers.clone(), self.selected_person.clone());
                        let peer_id = self.peers[self.selected_person].clone().to_string();
                        // Create a topic by combining and sorting peer IDs alphabetically
                        let mut peer_ids = vec![my_peer_id.clone(), peer_id.clone()];
                        peer_ids.sort(); // Sort alphabetically

                        let topic = gossipsub::IdentTopic::new(peer_ids.clone().join("_")); // Join with an appropriate separator

                        // Send the message to the selected peer
                        client.submit_message(input.clone(), topic.clone()).await;

                        let mut app = APP.lock().unwrap();
                        app.submit_private_message(peer_ids.join("_"));
                        // Clear input and reset state
                        app.input.clear();
                        app.character_index = 0;
                    } else if input.starts_with("!request file") {
                        let mut app = APP.lock().unwrap();
                        log::info!("Sending File Request");
                        let input_clone = app.input.clone();
                        let file: Vec<_> = input_clone.split(" ").collect();
                        let peer_id = self.peers[self.selected_person].clone();
                        client.send_request(file.get(file.len()-1).expect("").to_string(), peer_id).await;
                        app.input.clear();
                        app.character_index = 0;
                    } 

                }
            }
            KeyCode::Char('~') => {
                logger::info!("Tilda");

                if self.in_sidebar {
                    self.in_sidebar = false;
                    self.in_requests = true;
                } else if self.in_requests {
                    self.in_requests = false;
                } else {
                    self.in_sidebar = true;
                }
            }
            KeyCode::Char(to_insert) => {
                logger::info!("Pressed a Char");

                if !self.in_sidebar {
                    let mut app = APP.lock().unwrap();
                    app.enter_char(to_insert);
                }
            }
            KeyCode::Backspace => {
                if !self.in_sidebar {
                    let mut app = APP.lock().unwrap();
                    app.delete_char();
                }
            }
            KeyCode::Left => {
                if !self.in_sidebar {
                    let mut app = APP.lock().unwrap();
                    app.move_cursor_left();
                }
            }
            KeyCode::Right => {
                if !self.in_sidebar {
                    let mut app = APP.lock().unwrap();
                    app.move_cursor_right();
                }
            }
            KeyCode::Up => {
                if self.in_sidebar {
                    let user_count = self.usernames.len();
                    if user_count > 0 {
                        let i = match self.people_state.selected() {
                            Some(0) => user_count - 1, // Wrap to the last user
                            Some(i) => i - 1,
                            None => 0,
                        };
                        self.people_state.select(Some(i));
                    }
                } else if self.in_requests {
                    // Logic for moving up in the request list
                    let request_count = APP.lock().unwrap().current_requests.len();
                    if request_count > 0 {
                        let i = match self.request_state.selected() {
                            Some(0) => request_count - 1, // Wrap to the last request
                            Some(i) => i - 1,
                            None => 0,
                        };
                        self.request_state.select(Some(i));
                    }
                }
            }

            KeyCode::Down => {
                if self.in_sidebar {
                    let user_count = self.usernames.len();
                    if user_count > 0 {
                        let i = match self.people_state.selected() {
                            Some(i) if i >= user_count - 1 => 0, // Wrap to the first user
                            Some(i) => i + 1,
                            None => 0,
                        };
                        self.people_state.select(Some(i));
                    }
                } else if self.in_requests {
                    let request_count = APP.lock().unwrap().current_requests.len();
                    // Logic for moving down in the request list
                    if request_count > 0 {
                        let i = match self.request_state.selected() {
                            Some(i) if i >= request_count - 1 => 0, // Wrap to the first request
                            Some(i) => i + 1,
                            None => 0,
                        };
                        self.request_state.select(Some(i));
                    }
                }
            }

            KeyCode::Esc => {
                return Ok(true);
            }
            e => {logger::error!("{:?}", e)}
        }
        
        Ok(false)
    }

    fn select_person(&mut self) -> Option<String> {
        if let Some(selected) = self.people_state.selected() {
            if selected < self.peers.len() {
                let peer_id = self.peers[selected].to_string();
                logger::info!("Selected peer ID: {:?}", peer_id);
                self.selected_person = selected.clone();
            }
        }
        None
    }
}