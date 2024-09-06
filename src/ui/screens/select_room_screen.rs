use crate::state::APP; // Adjust the import to your actual application state location
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, List, ListItem, Borders},
};
use std::rc::Rc;
use crate::state::Screen::MainScreen;

/// Renders the list of chat rooms and the current room display.
///
/// Displays a list of available rooms and highlights the currently selected room.
/// Also shows the title of the current room in a bordered block.
pub fn render(frame: &mut Frame, chunk: Rc<[ratatui::layout::Rect]>) {
    let mut app = APP.lock().unwrap();

    let rooms_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1)])
        .split(chunk[1]);

    let room_items: Vec<ListItem> = app
        .rooms
        .iter()
        .map(|room| ListItem::new(room.clone()))
        .collect();

    let rooms_list = List::new(room_items)
        .block(Block::default().borders(Borders::ALL).title("Rooms"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">>");

    frame.render_stateful_widget(rooms_list, rooms_area[0], &mut app.room_state);

    let current_room = &app.rooms[app.current_room];
    let current_room_display = Block::default()
        .title(format!("Current Room: {}", current_room))
        .borders(Borders::ALL);
    frame.render_widget(current_room_display, rooms_area[0]);
}

/// Handles keyboard events for room navigation and selection.
///
/// Processes key inputs to navigate through the list of rooms and select a room.
/// Updates the application state accordingly. Returns `Ok(true)` if the Escape
/// key is pressed to exit the application, otherwise `Ok(false)`.
pub async fn handle_events(key: KeyEvent) -> Result<bool, std::io::Error> {
    let mut app = APP.lock().unwrap();
    match key.code {
        KeyCode::Enter => {
            if let Some(i) = app.room_state.selected() {
                app.current_room = i;
                app.current_screen = MainScreen;
            }
        }
        KeyCode::Up => {
            let i = match app.room_state.selected() {
                Some(i) => if i == 0 { app.rooms.len() - 1 } else { i - 1 },
                None => 0,
            };
            app.room_state.select(Some(i));
        }
        KeyCode::Down => {
            let i = match app.room_state.selected() {
                Some(i) => if i >= app.rooms.len() - 1 { 0 } else { i + 1 },
                None => 0,
            };
            app.room_state.select(Some(i));
        }
        KeyCode::Esc => return Ok(true),
        _ => {}
    }
    Ok(false)
}