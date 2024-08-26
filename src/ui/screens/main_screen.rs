use crate::state::APP;
use crate::network::network::Client;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    prelude::*,
    widgets::{Block, List, ListItem, Paragraph},
};
pub fn render(frame: &mut Frame) {
    let app = APP.lock().unwrap();
    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(1),
    ]);
    let [help_area, input_area, messages_area] = vertical.areas(frame.area());

    let (msg, style) = (
        vec!["SwapBytes ".bold()],
        Style::default(),
    );

    let text = Text::from(Line::from(msg)).patch_style(style);
    let help_message = Paragraph::new(text);
    frame.render_widget(help_message, help_area);

    let input = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::bordered().title("Input"));
    frame.render_widget(input, input_area);

    frame.set_cursor_position(Position {
        x: input_area.x + app.character_index as u16 + 1,
        y: input_area.y + 1,
    });

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .map(|m| ListItem::new(Line::from(Span::raw(m))))
        .collect();

    let messages = List::new(messages).block(Block::bordered().title("Messages"));
    frame.render_widget(messages, messages_area);
}

pub async fn handle_events(client: &mut Client) -> Result<bool, std::io::Error> {
    let mut app = APP.lock().unwrap();
    if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Enter => {
                        let message = format!("{}: {}", app.username.clone(), app.input.clone());
                        client.submit_message(message).await;
                        app.submit_message();
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
            }
        }
        Ok(false)
}