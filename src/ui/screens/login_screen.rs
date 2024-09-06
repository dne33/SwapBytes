use crate::state::APP;
use crate::network::network::Client;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, Paragraph},
};

/// Renders the login screen, including status and input fields.
///
/// Displays a status message based on peer connectivity and the input
/// area for username entry. Positions the cursor within the input area.
pub fn render(frame: &mut Frame) {
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
    ]);
    let [help_area, input_area] = vertical.areas(frame.area());

    let app = APP.lock().unwrap();
    let (msg, style) = if app.connected_peers > 0 {
        (vec!["SwapBytes".bold()], Style::default())
    } else {
        (vec!["Waiting for Peers to Connect".red()], Style::default().fg(Color::Red))
    };

    let text = Text::from(Line::from(msg)).patch_style(style);
    let help_message = Paragraph::new(text);
    frame.render_widget(help_message, help_area);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::bordered().title("Enter a Username"));
    frame.render_widget(input, input_area);

    frame.set_cursor_position(Position {
        x: input_area.x + app.character_index as u16 + 1,
        y: input_area.y + 1,
    });
    drop(app);
}

/// Handles keyboard events for the login screen.
///
/// Manages user input, updates application state, and interacts with
/// the server as needed. Returns `Ok(true)` if the Enter key triggers
/// an action, otherwise `Ok(false)`.
pub async fn handle_events(client: &mut Client, key: KeyEvent) -> Result<bool, std::io::Error> {
    let mut app = APP.lock().unwrap();
    match key.code {
        KeyCode::Enter => {
            if !app.input.is_empty() && app.connected_peers > 0 {
                app.username = app.input.clone();
                client.push_username(app.input.clone()).await;
                app.clear_input();
                return Ok(true);
            }
        },
        KeyCode::Char(to_insert) => app.enter_char(to_insert),
        KeyCode::Backspace => app.delete_char(),
        KeyCode::Left => app.move_cursor_left(),
        KeyCode::Right => app.move_cursor_right(),
        _ => {}
    }
    Ok(false)   
}
