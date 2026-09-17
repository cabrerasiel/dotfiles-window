//! Translates raw terminal input events into [`App`] state changes.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::{App, InputMode};

/// Reads the crossterm events and updates the state of [`App`].
pub fn handle_crossterm_events(app: &mut App) -> color_eyre::Result<()> {
    match event::read()? {
        // It's important to check KeyEventKind::Press to avoid handling key release events.
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
        InputMode::AddSection | InputMode::AddTodo | InputMode::AddKanbanColumns => {
            on_key_event_input(app, key)
        }
    }
}

fn on_key_event_normal(app: &mut App, key: KeyEvent) {
    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q') | KeyCode::Esc)
        | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
            app.quit();
        }
        (_, KeyCode::Tab | KeyCode::Left | KeyCode::Right) => {
            app.toggle_focus();
        }
        // h/l normally switch panels, but inside a kanban board (focus on
        // Todos) they move the cursor between columns without touching any
        // card — that's what H/L are for. Tab and the arrow keys remain a
        // fixed way to switch panels in every case.
        (_, KeyCode::Char('h')) => app.on_left(),
        (_, KeyCode::Char('l')) => app.on_right(),
        (_, KeyCode::Down | KeyCode::Char('j')) => app.next(),
        (_, KeyCode::Up | KeyCode::Char('k')) => app.previous(),
        (_, KeyCode::Char(' ')) => app.toggle_todo(),
        (_, KeyCode::Char('a')) => app.start_adding(),
        (_, KeyCode::Char('s')) => app.save_todos(),
        (_, KeyCode::Char('r')) => app.sync_linked_sections(),
        (_, KeyCode::Char('K')) => app.start_editing_columns(),
        (_, KeyCode::Char('H')) => app.move_card(-1),
        (_, KeyCode::Char('L')) => app.move_card(1),
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

        // Initial focus: the Sections panel.
        assert_eq!(app.focus, Focus::Sections);

        // Tab switches focus to the Todos panel.
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

        // Space -> toggle_todo
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

        for c in "Work".chars() {
            on_key_event(&mut app, KeyEvent::from(KeyCode::Char(c)));
        }
        on_key_event(&mut app, KeyEvent::from(KeyCode::Enter));

        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.sections.len(), 2);
        assert_eq!(app.sections[1].name, "Work");
    }

    #[test]
    fn test_cancel_add_todo_via_esc() {
        let mut app = App::new();
        app.toggle_focus(); // focus on Todos

        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('a')));
        assert_eq!(app.input_mode, InputMode::AddTodo);
        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('x')));
        on_key_event(&mut app, KeyEvent::from(KeyCode::Esc));

        assert_eq!(app.input_mode, InputMode::Normal);
        assert_eq!(app.sections[0].todos.len(), 3);
        // Esc while capturing input cancels; it must not quit the app.
        assert!(!app.should_quit);
    }

    #[test]
    fn test_kanban_via_keys() {
        let mut app = App::new();

        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('K')));
        assert_eq!(app.input_mode, InputMode::AddKanbanColumns);

        for c in "A,B,C".chars() {
            on_key_event(&mut app, KeyEvent::from(KeyCode::Char(c)));
        }
        assert_eq!(app.input_buffer, "A,B,C");

        on_key_event(&mut app, KeyEvent::from(KeyCode::Enter));

        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.sections[0].columns.is_some());
    }

    #[test]
    fn test_kanban_key_without_sections_shows_message() {
        let mut app = App::new();
        app.sections.clear();

        on_key_event(&mut app, KeyEvent::from(KeyCode::Char('K')));

        assert_eq!(app.input_mode, InputMode::Normal);
        assert!(app.message.unwrap().contains("No section is selected"));
    }
}
