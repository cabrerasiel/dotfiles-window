use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{App, InputMode};

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
    match app.input_mode {
        InputMode::Normal => on_key_event_normal(app, key),
        InputMode::AddSection | InputMode::AddTodo => on_key_event_input(app, key),
    }
}

fn on_key_event_normal(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q') | KeyCode::Esc)
        | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
            app.quit();
        }
        (
            _,
            KeyCode::Tab | KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l'),
        ) => {
            app.toggle_focus();
        }
        (_, KeyCode::Down | KeyCode::Char('j')) => app.next(),
        (_, KeyCode::Up | KeyCode::Char('k')) => app.previous(),
        (_, KeyCode::Char(' ')) => app.toggle_todo(),
        (_, KeyCode::Char('a')) => app.start_adding(),
        (_, KeyCode::Char('s')) => app.save_todos(),
        _ => {}
    }
}

fn on_key_event_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => app.confirm_input(),
        KeyCode::Esc => app.cancel_input(),
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Char(c) => app.input_buffer.push(c),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Focus;

    #[test]
    fn test_navigation_and_toggle() {
        let mut app = App::new();

        // Foco inicial: panel de Secciones
        assert_eq!(app.focus, Focus::Sections);

        // Tab cambia el foco al panel de Todos
        on_key_event(&mut app, KeyEvent::from(KeyCode::Tab));
        assert_eq!(app.focus, Focus::Todos);

        // Down / j -> next_todo
        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('j')));
        assert_eq!(app.selected_todo, 1);

        on_key_event(&mut app, KeyEvent::from(KeyCode::Down));
        assert_eq!(app.selected_todo, 2);

        // Up / k -> previous_todo
        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('k')));
        assert_eq!(app.selected_todo, 1);

        on_key_event(&mut app, KeyEvent::from(KeyCode::Up));
        assert_eq!(app.selected_todo, 0);

        // Espacio -> toggle_todo
        assert!(!app.sections[0].todos[0].is_done);
        on_key_event(&mut app, KeyEvent::from(KeyCode::Char(' ')));
        assert!(app.sections[0].todos[0].is_done);

        // q -> should_quit
        assert!(!app.should_quit);
        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('q')));
        assert!(app.should_quit);
    }

    #[test]
    fn test_add_section_via_keys() {
        let mut app = App::new();

        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('a')));
        assert_eq!(app.input_mode, InputMode::AddSection);

        for c in "Trabajo".chars() {
            on_key_event(&mut app, KeyEvent::from(KeyCode::Char(c)));
        }
        on_key_event(&mut app, KeyEvent::from(KeyCode::Enter));

        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.sections.len(), 2);
        assert_eq!(app.sections[1].name, "Trabajo");
    }

    #[test]
    fn test_cancel_add_todo_via_esc() {
        let mut app = App::new();
        app.toggle_focus(); // foco en Todos

        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('a')));
        assert_eq!(app.input_mode, InputMode::AddTodo);
        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('x')));
        on_key_event(&mut app, KeyEvent::from(KeyCode::Esc));

        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.sections[0].todos.len(), 3);
        // Esc en modo entrada cancela, no debe cerrar la aplicación
        assert!(!app.should_quit);
    }
}
