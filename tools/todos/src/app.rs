use crate::models::todos::Todo;
use ratatui::DefaultTerminal;

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub selected_index: usize,
    pub todos: Vec<Todo>,
}

impl App {
    /// Construct a new instance of [`App`] with default todos.
    pub fn new() -> Self {
        Self::with_todos(Self::default_todos())
    }

    /// Construct a new instance of [`App`] with custom todos.
    pub fn with_todos(todos: Vec<Todo>) -> Self {
        Self {
            todos,
            selected_index: 0,
            should_quit: false,
        }
    }

    pub fn default_todos() -> Vec<Todo> {
        vec![
            Todo::new("Aprender Ratatui profundamente".to_string()),
            Todo::new("Configurar persistencia con Serde".to_string()),
            Todo::new("Hacer ejercicio por la tarde".to_string()),
        ]
    }

    /// Run the application's main loop.
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| crate::ui::render(self, frame))?;
            crate::handler::handle_crossterm_events(self)?;
        }
        Ok(())
    }

    pub fn next_todo(&mut self) {
        if !self.todos.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.todos.len();
        }
    }

    pub fn previous_todo(&mut self) {
        if !self.todos.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.todos.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn toggle_todo(&mut self) {
        if let Some(todo) = self.todos.get_mut(self.selected_index) {
            todo.is_done = !todo.is_done;
        }
    }

    /// Set should_quit to true to quit the application.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
