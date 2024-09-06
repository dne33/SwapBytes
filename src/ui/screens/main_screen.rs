use crate::state::APP;
use crate::network::network::Client;
use libp2p::gossipsub;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, List, ListItem, Paragraph},
};
use std::rc::Rc;
use crate::logger;

pub fn render(frame: &mut Frame, chunk: Rc<[ratatui::layout::Rect]>) {
    let app = APP.lock().unwrap();
    let vertical = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
    ]);
    let [input_area, messages_area] = vertical.areas(chunk[1]);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::bordered().title("Input"));
    frame.render_widget(input, input_area);

    frame.set_cursor_position(Position {
        x: input_area.x + app.character_index as u16 + 1,
        y: input_area.y + 1,
    });

    let current_room_name = app.rooms.get(app.current_room);

    let messages: Vec<ListItem> = if let Some(room_name) = current_room_name {
        app.public_messages
            .get(room_name)
            .unwrap() // Use an empty vector if no messages are found
            .iter()
            .map(|m| ListItem::new(Line::from(Span::raw(m))))
            .collect()
    } else {
        Vec::new() // If no room is selected, return an empty vector
    };

    let current_room = &app.rooms[app.current_room];
    let messages = List::new(messages).block(Block::bordered().title(format!("Current Room: {}", current_room)));
    frame.render_widget(messages, messages_area);
}

pub async fn handle_events(client: &mut Client, key: KeyEvent) -> Result<bool, std::io::Error> {
    let mut app = APP.lock().unwrap();
    match key.code {
        KeyCode::Enter => {
            if app.input.clone().starts_with("!create room ") {
                let chat_name: &str = &app.input.clone()[13..app.input.clone().len()];
                logger::info!("Attempting to create room: {}", chat_name);
                let chat_name_len = chat_name.len();
                if chat_name_len <= 64 && chat_name_len > 0 {
                    client.create_room(chat_name.to_string()).await;
                    app.input.clear();
                    app.character_index = 0;
                } else {
                    logger::info!("Failed to add chat room name, name too long")
                }
            } else {
                let message = format!("{}: {}", app.username.clone(), app.input.clone());
                app.submit_public_room_message();
                let room_name = app.rooms.get(app.current_room).unwrap_or(&"global".to_string()).clone();
                let topic = gossipsub::IdentTopic::new(room_name);
                client.submit_message(message, topic).await;
            }
            
        }
        KeyCode::Char(to_insert) => {
            app.enter_char(to_insert);
        }
        KeyCode::Backspace => {
            app.delete_char();
        }
        KeyCode::Left => {
            app.move_cursor_left();
        }
        KeyCode::Right => {
            app.move_cursor_right();
        },
        KeyCode::Tab => {
        },
        KeyCode::Esc => {
            return Ok(true);
        }
        _ => {}
    }
        
    Ok(false)
}