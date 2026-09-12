use crate::models::todos::{Section, Todo};
use ratatui::DefaultTerminal;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Panel con el foco de navegación actual.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Sections,
    Todos,
}

/// Modo de entrada de texto: normal (navegación) o capturando un nombre nuevo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    AddSection,
    AddTodo,
}

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub sections: Vec<Section>,
    pub selected_section: usize,
    pub selected_todo: usize,
    pub focus: Focus,
    pub file_path: Option<PathBuf>,
    pub message: Option<String>,
    pub input_mode: InputMode,
    pub input_buffer: String,
}

impl App {
    /// Construct a new instance of [`App`] with default sections.
    pub fn new() -> Self {
        Self::with_sections(Self::default_sections())
    }

    /// Construct a new instance of [`App`] with custom sections.
    pub fn with_sections(sections: Vec<Section>) -> Self {
        Self {
            sections,
            ..Default::default()
        }
    }

    pub fn default_sections() -> Vec<Section> {
        vec![Section {
            id: Uuid::new_v4(),
            name: "General".to_string(),
            todos: vec![
                Todo::new("Aprender Ratatui profundamente".to_string()),
                Todo::new("Configurar persistencia con Serde".to_string()),
                Todo::new("Hacer ejercicio por la tarde".to_string()),
            ],
        }]
    }

    /// Ruta por defecto donde se guardan las tareas dentro de la carpeta del usuario (`$HOME/.todos.json`).
    pub fn default_save_path() -> PathBuf {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".todos.json")
    }

    /// Deserializa secciones desde JSON. Si el contenido es del formato antiguo (una lista
    /// plana de tareas), lo migra envolviéndolo en una única sección "General".
    pub fn parse_sections(content: &str) -> Vec<Section> {
        if let Ok(sections) = serde_json::from_str::<Vec<Section>>(content) {
            return sections;
        }
        match serde_json::from_str::<Vec<Todo>>(content) {
            Ok(todos) if !todos.is_empty() => vec![Section {
                id: Uuid::new_v4(),
                name: "General".to_string(),
                todos,
            }],
            _ => Vec::new(),
        }
    }

    /// Run the application's main loop.
    pub fn run(&mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| crate::ui::render(self, frame))?;
            crate::handler::handle_crossterm_events(self)?;
        }
        Ok(())
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Sections => Focus::Todos,
            Focus::Todos => Focus::Sections,
        };
    }

    pub fn selected_section(&self) -> Option<&Section> {
        self.sections.get(self.selected_section)
    }

    /// Avanza la selección en el panel con foco actual.
    pub fn next(&mut self) {
        match self.focus {
            Focus::Sections => self.next_section(),
            Focus::Todos => self.next_todo(),
        }
    }

    /// Retrocede la selección en el panel con foco actual.
    pub fn previous(&mut self) {
        match self.focus {
            Focus::Sections => self.previous_section(),
            Focus::Todos => self.previous_todo(),
        }
    }

    fn next_section(&mut self) {
        if !self.sections.is_empty() {
            self.selected_section = (self.selected_section + 1) % self.sections.len();
            self.selected_todo = 0;
        }
    }

    fn previous_section(&mut self) {
        if !self.sections.is_empty() {
            self.selected_section = if self.selected_section == 0 {
                self.sections.len() - 1
            } else {
                self.selected_section - 1
            };
            self.selected_todo = 0;
        }
    }

    fn next_todo(&mut self) {
        if let Some(section) = self.sections.get(self.selected_section)
            && !section.todos.is_empty()
        {
            self.selected_todo = (self.selected_todo + 1) % section.todos.len();
        }
    }

    fn previous_todo(&mut self) {
        if let Some(section) = self.sections.get(self.selected_section)
            && !section.todos.is_empty()
        {
            self.selected_todo = if self.selected_todo == 0 {
                section.todos.len() - 1
            } else {
                self.selected_todo - 1
            };
        }
    }

    pub fn toggle_todo(&mut self) {
        if self.focus != Focus::Todos {
            return;
        }
        if let Some(section) = self.sections.get_mut(self.selected_section)
            && let Some(todo) = section.todos.get_mut(self.selected_todo)
        {
            todo.is_done = !todo.is_done;
        }
    }

    /// Empieza a capturar texto para agregar una sección o una tarea, según el panel con foco.
    pub fn start_adding(&mut self) {
        if self.focus == Focus::Todos && self.sections.is_empty() {
            self.message = Some("Crea primero una sección (Tab, luego 'a')".to_string());
            return;
        }
        self.input_buffer.clear();
        self.input_mode = match self.focus {
            Focus::Sections => InputMode::AddSection,
            Focus::Todos => InputMode::AddTodo,
        };
    }

    /// Confirma el texto capturado, creando la sección o tarea correspondiente.
    pub fn confirm_input(&mut self) {
        let text = self.input_buffer.trim().to_string();
        if !text.is_empty() {
            match self.input_mode {
                InputMode::AddSection => {
                    self.sections.push(Section::new(text));
                    self.selected_section = self.sections.len() - 1;
                    self.selected_todo = 0;
                }
                InputMode::AddTodo => {
                    if let Some(section) = self.sections.get_mut(self.selected_section) {
                        section.todos.push(Todo::new(text));
                        self.selected_todo = section.todos.len() - 1;
                    }
                }
                InputMode::Normal => {}
            }
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    /// Cancela la captura de texto en curso sin crear nada.
    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    /// Serializa las secciones y las guarda en `file_path`, actualizando `message` con el resultado.
    pub fn save_todos(&mut self) {
        let Some(path) = self.file_path.clone() else {
            self.message =
                Some("No hay archivo de destino: usa `todos <archivo.json>`".to_string());
            return;
        };

        self.message = Some(match serde_json::to_string_pretty(&self.sections) {
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
        app.focus = Focus::Todos;
        app.toggle_todo();
        app.save_todos();

        assert!(app.message.unwrap().contains("Guardado"));

        let content = fs::read_to_string(&file_path).unwrap();
        let saved: Vec<Section> = serde_json::from_str(&content).unwrap();
        assert_eq!(saved.len(), app.sections.len());
        assert!(saved[0].todos[0].is_done);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_sections_migrates_flat_todo_list() {
        let old_format =
            r#"[{"id":"397fabd8-38c5-4b44-a389-b3db35b1d044","text":"algo","is_done":false}]"#;
        let sections = App::parse_sections(old_format);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].name, "General");
        assert_eq!(sections[0].todos[0].text, "algo");
    }

    #[test]
    fn test_add_section_and_todo() {
        let mut app = App::new();
        app.start_adding();
        app.input_buffer = "Trabajo".to_string();
        app.confirm_input();
        assert_eq!(app.sections.len(), 2);
        assert_eq!(app.sections[1].name, "Trabajo");
        assert_eq!(app.selected_section, 1);

        app.toggle_focus();
        app.start_adding();
        app.input_buffer = "Nueva tarea".to_string();
        app.confirm_input();
        assert_eq!(app.sections[1].todos.len(), 1);
        assert_eq!(app.sections[1].todos[0].text, "Nueva tarea");
    }
}
