use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::App;

/// Reads the crossterm events and updates the state of [`App`].
pub fn handle_crossterm_events(app: &mut App) -> color_eyre::Result<()> {
    match event::read()? {
        // it's important to check KeyEventKind::Press to avoid handling key release events
        Event::Key(key) if key.kind == KeyEventKind::Press => on_key_event(app, key),
        Event::Mouse(_) => {}
        Event::Resize(_, _) => {}
        _ => {}
    }
    Ok(())
}

/// Handles the key events and updates the state of [`App`].
pub fn on_key_event(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q') | KeyCode::Esc)
        | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
            app.quit();
        }
        (_, KeyCode::Down | KeyCode::Char('j')) => app.next_todo(),
        (_, KeyCode::Up | KeyCode::Char('k')) => app.previous_todo(),
        (_, KeyCode::Char(' ')) => app.toggle_todo(),
        (_, KeyCode::Char('s')) => app.save_todos(),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_events() {
        let mut app = App::new();

        // Down / j -> next_todo
        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('j')));
        assert_eq!(app.selected_index, 1);

        on_key_event(&mut app, KeyEvent::from(KeyCode::Down));
        assert_eq!(app.selected_index, 2);

        // Up / k -> previous_todo
        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('k')));
        assert_eq!(app.selected_index, 1);

        on_key_event(&mut app, KeyEvent::from(KeyCode::Up));
        assert_eq!(app.selected_index, 0);

        // Space -> toggle_todo
        assert!(!app.todos[0].is_done);
        on_key_event(&mut app, KeyEvent::from(KeyCode::Char(' ')));
        assert!(app.todos[0].is_done);

        // q -> should_quit
        assert!(!app.should_quit);
        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('q')));
        assert!(app.should_quit);
    }
}
