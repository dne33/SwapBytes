// dm_screen.rs

use ratatui::{
    style::{Modifier, Style, Color},
    text::{Line, Span, Text},
    layout::{Constraint, Layout, Position},
    Frame,
    prelude::*,
    widgets::{List, ListItem, Paragraph, ListState, Block, Borders},
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
};
use crate::network::network::Client;
use crate::logger;
use crate::APP;
use std::collections::HashMap;
use libp2p::PeerId;

pub struct DmScreen {
    pub input: String,
    pub messages: Vec<String>,
    pub people: Vec<String>,
    pub people_state: ListState,
    pub selected_person: usize,
    pub in_sidebar: bool,
    pub character_index: usize,
}

impl DmScreen {
    pub fn new() -> Self {
        let mut people_state = ListState::default();
        people_state.select(Some(0));
        Self {
            input: String::new(),
            messages: Vec::new(),
            people: vec![
                "Alice".to_string(),
                "Bob".to_string(),
                "Charlie".to_string(),
                "Diana".to_string(),
                "Eve".to_string(),
            ],
            people_state,
            selected_person: 0,
            in_sidebar: false,
            character_index: 0,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, usernames: HashMap<String, String>, peers: Vec<PeerId>) {
        let people = 0;
        let horizontal = Layout::horizontal([
            Constraint::Length(30),
            Constraint::Min(1),
        ]);
        let [sidebar_area, main_area] = horizontal.areas(frame.area());

        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, messages_area] = vertical.areas(main_area);

        // Help message
        let (msg, style) = (vec!["SwapBytes ".bold()], Style::default());
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        // Input area
        let input_style = if self.in_sidebar {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Yellow)
        };

        let input = Paragraph::new(self.input.as_str())
            .style(input_style)
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);

        if !self.in_sidebar {
            frame.set_cursor_position(Position {
                x: input_area.x + self.character_index as u16 + 1,
                y: input_area.y + 1,
            });
        }

        // Messages area
        let message_store: Vec<ListItem> = self
            .messages
            .iter()
            .map(|m| ListItem::new(Line::from(Span::raw(m))))
            .collect();
        let message_store = List::new(message_store).block(Block::bordered().title("Messages"));
        frame.render_widget(message_store, messages_area);

        // let peer_items: Vec<ListItem> = self
        //     .people
        //     .iter()
        //     .map(|users| ListItem::new(users.clone()))
        //     .collect();
        
        let peer_list: Vec<String> = peers.iter().map(|peer_id| format!("{}", peer_id.to_string())).collect();  
        logger::info!("1");
        let peer_items: Vec<ListItem> = peer_list
        .iter()
        .filter_map(|peer| {
            usernames.get(peer).map(|username| ListItem::new(format!("{}", username)))
        })
        .collect();
        logger::info!("2");

        // Sidebar (people list)
        let people_list = List::new(peer_items)
            .block(Block::default().borders(Borders::ALL).title("People"))
            .highlight_style(if self.in_sidebar {
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)
            } else {
                Style::default().add_modifier(Modifier::REVERSED)
            })
            .highlight_symbol(">>");
        frame.render_stateful_widget(people_list, sidebar_area, &mut self.people_state);    
    }

    pub async fn handle_events(&mut self) -> Result<bool, std::io::Error> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {
                        logger::info!("Enter pressed");

                        if self.in_sidebar {
                            logger::info!(" Sidebar");

                            self.select_person();
                        } else {
                            logger::info!(" Submit");
                            
                            // let app = APP.lock().unwrap();
                            // let message = format!("{}: {}", app.username.clone(), self.input.clone());
                            // client.submit_message(message).await;
                            self.submit_message();
                        }
                    }
                    KeyCode::Char('~') => {
                        logger::info!("Tilda");

                        self.in_sidebar = !self.in_sidebar;
                    }
                    KeyCode::Char(to_insert) => {
                        logger::info!("Pressed a Char");

                        if !self.in_sidebar {
                            self.enter_char(to_insert);
                        }
                    }
                    KeyCode::Backspace => {
                        if !self.in_sidebar {
                            self.delete_char();
                        }
                    }
                    KeyCode::Left => {
                        if !self.in_sidebar {
                            self.move_cursor_left();
                        }
                    }
                    KeyCode::Right => {
                        if !self.in_sidebar {
                            self.move_cursor_right();
                        }
                    }
                    KeyCode::Up => {
                        if self.in_sidebar {
                        let i = match self.people_state.selected() {
                            Some(i) => {
                                if i == 0 {
                                    self.people.len() - 1
                                } else {
                                    i - 1
                                }
                            }
                            None => 0,
                        };
                            self.people_state.select(Some(i));
                        }
             
                    }
                    KeyCode::Down => {
                        if self.in_sidebar {
                            let i = match self.people_state.selected() {
                                Some(i) => {
                                    if i >= self.people.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            self.people_state.select(Some(i));
                        }
                        
                    }
                    KeyCode::Esc => {
                        return Ok(true);
                    }
                    e => {logger::error!("{:?}", e)}
                }
            }
        }
        Ok(false)
    }

    fn submit_message(&mut self) {
        logger::info!("submit message");
        self.messages.push(self.input.clone());
        self.input.clear();
        self.character_index = 0;
    }

    fn enter_char(&mut self, c: char) {
        logger::info!("enter char");

        self.input.push(c);
        self.character_index += 1;
    }

    fn delete_char(&mut self) {
        if self.character_index > 0 {
            self.input.pop();
            self.character_index -= 1;
        }
    }

    fn move_cursor_left(&mut self) {
        if self.character_index > 0 {
            self.character_index -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.character_index < self.input.len() {
            self.character_index += 1;
        }
    }

    fn select_person(&self) {
        // Implement what should happen when a person is selected
    }
}
