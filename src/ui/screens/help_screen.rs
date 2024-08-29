use crate::state::APP;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyEvent},
    prelude::*,
    widgets::{Block, Borders},
};
use std::rc::Rc;

pub fn render(frame: &mut Frame, chunk: Rc<[ratatui::layout::Rect]>) {
    let block = Block::default().title("Help").borders(Borders::ALL);
    frame.render_widget(block, chunk[1]);
    // Add more content to the help screen as needed
}

pub async fn handle_events(key: KeyEvent) -> Result<bool, std::io::Error> {
    let mut app = APP.lock().unwrap();
    
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
    Ok(false)
}