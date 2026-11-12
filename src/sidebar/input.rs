//! Terminal events become messages.

use ratatui::crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};

use super::update::{Action, Msg};
use crate::jump::Direction;

/// `preview`: the key that opens the preview popup, which comes before the fixed keys.
pub fn message(event: &Event, popup: bool, preview: Option<char>) -> Option<Msg> {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press && preview_key(key, preview) => {
            Some(Msg::Action(Action::Preview))
        }
        Event::Key(key) if key.kind == KeyEventKind::Press => action(key, popup).map(Msg::Action),
        Event::Mouse(mouse) if mouse.kind == MouseEventKind::Down(MouseButton::Left) => {
            Some(Msg::Clicked { y: mouse.row })
        }
        Event::Resize(_, height) => Some(Msg::Resized { height: *height }),
        // The preview shows only with the focus; reload to show or hide it now (tmux `focus-events on`).
        Event::FocusGained | Event::FocusLost => Some(Msg::Action(Action::Reload)),
        _ => None,
    }
}

/// The popup key, unmodified except for the Shift of a capital letter.
fn preview_key(key: &KeyEvent, preview: Option<char>) -> bool {
    let plain = key.modifiers.difference(KeyModifiers::SHIFT).is_empty();
    plain && preview.is_some_and(|c| key.code == KeyCode::Char(c))
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

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
    }

    #[test]
    fn maps_keys_to_actions() {
        assert_eq!(
            message(&key(KeyCode::Char('j')), false, None),
            Some(Msg::Action(Action::Move(1)))
        );
        assert_eq!(
            message(&key(KeyCode::Enter), false, None),
            Some(Msg::Action(Action::Activate))
        );
        assert_eq!(message(&key(KeyCode::Char('x')), false, None), None);
    }

    #[test]
    fn escape_closes_only_the_popup() {
        assert_eq!(
            message(&key(KeyCode::Esc), true, None),
            Some(Msg::Action(Action::Quit))
        );
        assert_eq!(message(&key(KeyCode::Esc), false, None), None);
    }

    #[test]
    fn the_preview_key_opens_the_popup_only_when_set() {
        let space = key(KeyCode::Char(' '));
        let preview = Some(Msg::Action(Action::Preview));
        assert_eq!(message(&space, false, Some(' ')), preview);
        assert_eq!(message(&space, false, None), None);
        assert_eq!(message(&key(KeyCode::Char('v')), false, Some('v')), preview);
        let modified = |modifiers| Event::Key(KeyEvent::new(KeyCode::Char(' '), modifiers));
        assert_eq!(
            message(&modified(KeyModifiers::CONTROL), false, Some(' ')),
            None
        );
        assert_eq!(
            message(&modified(KeyModifiers::ALT), false, Some(' ')),
            None
        );
    }
}
