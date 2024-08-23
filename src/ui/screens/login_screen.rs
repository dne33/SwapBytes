use crate::state::{APP, Screen};
use crate::network::network::Client;
use ratatui::{
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::*,
    widgets::{Block, List, ListItem, Paragraph, Borders},
};

use crate::logger;

pub fn render(frame: &mut Frame) {
    let mut app = APP.lock().unwrap();

    // Determine the layout
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
    ]);
    let [help_area, input_area] = vertical.areas(frame.area());
    // Display message based on connection status   
    let (msg, style) = if app.connected_peers.clone() > 0 {
        (vec!["SwapBytes".bold()], Style::default())
    } else {
        (vec!["Waiting for Peers to Connect".red()], Style::default().fg(Color::Red))
    };

    let text = Text::from(Line::from(msg)).patch_style(style);
    let help_message = Paragraph::new(text);
    frame.render_widget(help_message, help_area);

    // Render input area for username
    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::bordered().title("Enter a Username"));
    frame.render_widget(input, input_area);

    // Set cursor position
    frame.set_cursor_position(Position {
        x: input_area.x + app.character_index as u16 + 1,
        y: input_area.y + 1,
    });
    
}

pub async fn handle_events() -> Result<bool, std::io::Error> {
    let mut app = APP.lock().unwrap();
    if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {
                        if !app.input.clone().is_empty() && app.connected_peers > 0 {
                            logger::info!("Should Move through");
                            app.username = app.input.clone();
                            app.clear_input();
                            return Ok(true);
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
                    _ => {}
                }
            }
        }
        Ok(false)
}