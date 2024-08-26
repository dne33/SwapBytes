use crate::state::APP; // Adjust the import to your actual application state location
use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
    },
    prelude::*,
    widgets::{Block, List, ListItem, Borders},
};

// Function to render the room list
pub fn render(frame: &mut Frame) {
    let mut app = APP.lock().unwrap();

    // Define the layout for the rooms
    let rooms_area = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Min(1)])
        .split(frame.area());

    // Create the list of room items
    let room_items: Vec<ListItem> = app
        .rooms
        .iter()
        .map(|room| ListItem::new(room.clone()))
        .collect();

    // Create the List widget
    let rooms_list = List::new(room_items)
        .block(Block::default().borders(Borders::ALL).title("Rooms"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>");

    // Render the list widget with state
    frame.render_stateful_widget(rooms_list, rooms_area[0], &mut app.room_state);

   
    let current_room = &app.rooms[app.current_room];
    let current_room_display = Block::default()
        .title(format!("Current Room: {}", current_room))
        .borders(Borders::ALL);
    frame.render_widget(current_room_display, rooms_area[0]);
    
}

// Function to handle key events for room navigation
pub async fn handle_events() -> Result<bool, std::io::Error> {
    let mut app = APP.lock().unwrap();

    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Enter => {
                    let _i = match app.room_state.selected() {
                        Some(i) => {
                            app.current_room = i;
                        }
                        None => {},
                    };
                }
                KeyCode::Up => {
                    let i = match app.room_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                app.rooms.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    app.room_state.select(Some(i));
                }
                KeyCode::Down => {
                    let i = match app.room_state.selected() {
                        Some(i) => {
                            if i >= app.rooms.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    app.room_state.select(Some(i));
                }
                KeyCode::Esc => {
                    return Ok(true); // Exit the application
                }
                _ => {}
            }
        }
    }
    Ok(false)
}
