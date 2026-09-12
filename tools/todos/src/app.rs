use crate::models::todos::Todo;
use ratatui::DefaultTerminal;
use std::fs;
use std::path::PathBuf;

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub selected_index: usize,
    pub todos: Vec<Todo>,
    pub file_path: Option<PathBuf>,
    pub message: Option<String>,
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
            file_path: None,
            message: None,
        }
    }

    pub fn default_todos() -> Vec<Todo> {
        vec![
            Todo::new("Aprender Ratatui profundamente".to_string()),
            Todo::new("Configurar persistencia con Serde".to_string()),
            Todo::new("Hacer ejercicio por la tarde".to_string()),
        ]
    }

    /// Ruta por defecto donde se guardan las tareas dentro de la carpeta del usuario (`$HOME/.todos.json`).
    pub fn default_save_path() -> PathBuf {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".todos.json")
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

    /// Serializa las tareas y las guarda en `file_path`, actualizando `message` con el resultado.
    pub fn save_todos(&mut self) {
        let Some(path) = self.file_path.clone() else {
            self.message =
                Some("No hay archivo de destino: usa `todos <archivo.json>`".to_string());
            return;
        };

        self.message = Some(match serde_json::to_string_pretty(&self.todos) {
            Ok(json) => match fs::write(&path, json) {
                Ok(()) => format!("Guardado en {}", path.display()),
                Err(err) => format!("Error al guardar: {err}"),
            },
            Err(err) => format!("Error al serializar: {err}"),
        });
    }

    /// Set should_quit to true to quit the application.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_todos_without_file_path() {
        let mut app = App::new();
        app.save_todos();
        assert!(app.message.unwrap().contains("No hay archivo de destino"));
    }

    #[test]
    fn test_save_todos_writes_json() {
        let dir = std::env::temp_dir().join("todos_test_save");
        let _ = fs::create_dir_all(&dir);
        let file_path = dir.join("todos.json");

        let mut app = App::new();
        app.file_path = Some(file_path.clone());
        app.toggle_todo();
        app.save_todos();

        assert!(app.message.unwrap().contains("Guardado"));

        let content = fs::read_to_string(&file_path).unwrap();
        let saved: Vec<Todo> = serde_json::from_str(&content).unwrap();
        assert_eq!(saved.len(), app.todos.len());
        assert!(saved[0].is_done);

        let _ = fs::remove_dir_all(&dir);
    }
}
