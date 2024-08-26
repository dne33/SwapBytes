use crate::state::APP;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    prelude::*,
    widgets::{Block, Borders},
};

pub fn render(frame: &mut Frame) {
    let block = Block::default().title("Help").borders(Borders::ALL);
    frame.render_widget(block, frame.area());
    // Add more content to the help screen as needed
}

pub async fn handle_events() -> Result<bool, std::io::Error> {
    let mut app = APP.lock().unwrap();
    if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {}
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
            }
        }
        Ok(false)
}