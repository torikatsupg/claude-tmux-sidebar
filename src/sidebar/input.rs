//! Terminal events become messages.

use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind,
};

use super::update::{Action, Msg};
use crate::jump::Direction;

pub fn message(event: &Event, popup: bool) -> Option<Msg> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => action(key, popup).map(Msg::Action),
        Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
            Some(Msg::Clicked { y: mouse.row })
        }
        Event::Resize(_, height) => Some(Msg::Resized { height: *height }),
        _ => None,
    }
}

fn action(key: &KeyEvent, popup: bool) -> Option<Action> {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Action::Move(1)),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::Move(-1)),
        KeyCode::Char('n') => Some(Action::Attention(Direction::Next)),
        KeyCode::Char('p') => Some(Action::Attention(Direction::Prev)),
        KeyCode::Char('g') | KeyCode::Home => Some(Action::First),
        KeyCode::Char('G') | KeyCode::End => Some(Action::Last),
        KeyCode::Enter | KeyCode::Char('l') => Some(Action::Activate),
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Esc if popup => Some(Action::Quit),
        KeyCode::Char('r') => Some(Action::Reload),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn maps_keys_to_actions() {
        assert_eq!(
            message(&key(KeyCode::Char('j')), false),
            Some(Msg::Action(Action::Move(1)))
        );
        assert_eq!(
            message(&key(KeyCode::Enter), false),
            Some(Msg::Action(Action::Activate))
        );
        assert_eq!(message(&key(KeyCode::Char('x')), false), None);
    }

    #[test]
    fn escape_closes_only_the_popup() {
        assert_eq!(
            message(&key(KeyCode::Esc), true),
            Some(Msg::Action(Action::Quit))
        );
        assert_eq!(message(&key(KeyCode::Esc), false), None);
    }
}
