//! Application state and behavior: the [`App`] struct holds everything the
//! UI renders and the key handler mutates.

use crate::models::todos::{Section, SectionSource, Todo};
use ratatui::DefaultTerminal;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Reflects a done/not-done boolean in Reminders (the only external system
/// that understands that simple state). Only called by `toggle_todo`, which
/// already blocks kanban sections before reaching this — and a Jira section
/// is always a kanban section (see `Section::boards`), so the Jira arm here
/// is never actually reached; Jira's real sync goes through
/// `sync_external_column` instead.
fn sync_external_done(
    source: &SectionSource,
    external_id: Option<&str>,
    done: bool,
) -> color_eyre::Result<()> {
    let Some(id) = external_id else {
        return Ok(());
    };
    match source {
        SectionSource::Reminders { .. } => crate::eventkit_sync::set_reminder_completed(id, done),
        SectionSource::Jira { .. } | SectionSource::Local | SectionSource::Calendar { .. } => {
            Ok(())
        }
    }
}

/// Reflects, in the relevant external system, that a card moved to the
/// column named `column_name`. Jira transitions the issue to the status
/// with that exact name; Reminders only understands done/not-done, so it
/// uses `is_last` (whether the target column is the last one) as an
/// approximation. Does nothing for local or Calendar sections.
fn sync_external_column(
    source: &SectionSource,
    external_id: Option<&str>,
    column_name: &str,
    is_last: bool,
) -> color_eyre::Result<()> {
    let Some(id) = external_id else {
        return Ok(());
    };
    match source {
        SectionSource::Reminders { .. } => {
            crate::eventkit_sync::set_reminder_completed(id, is_last)
        }
        SectionSource::Jira { .. } => crate::jira_sync::set_issue_status(id, column_name),
        SectionSource::Local | SectionSource::Calendar { .. } => Ok(()),
    }
}

/// Which panel currently has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Focus {
    #[default]
    Sections,
    Todos,
}

/// Text-input mode: normal navigation, or capturing a new name/value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    AddSection,
    AddTodo,
    AddKanbanColumns,
}

/// The main application state: everything the UI renders and the key
/// handler mutates.
#[derive(Debug, Default)]
pub struct App {
    pub should_quit: bool,
    pub sections: Vec<Section>,
    pub selected_section: usize,
    /// Selection within a plain-list (non-kanban) section. For kanban
    /// sections, the active card is instead derived from `kanban_board` /
    /// `kanban_column` / `kanban_item` — see `current_kanban_todo_index`.
    pub selected_todo: usize,
    /// Current board (stacked row) within a multi-board (Jira) section.
    /// Always `0` for single-board sections.
    pub kanban_board: usize,
    /// Current column within the board. The cursor can rest on an empty
    /// column, lazygit-style — it doesn't depend on any card being there.
    pub kanban_column: usize,
    /// Position of the active card within the current (board, column) cell,
    /// used when that cell holds more than one card.
    pub kanban_item: usize,
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
                Todo::new("Learn Ratatui in depth".to_string()),
                Todo::new("Set up persistence with Serde".to_string()),
                Todo::new("Exercise in the afternoon".to_string()),
            ],
            source: SectionSource::Local,
            columns: None,
            boards: None,
        }]
    }

    /// Inserts or replaces, within `sections`, the synced section (Reminders/
    /// Calendar/Jira) with the same `source` as `new_section`, keeping its
    /// position; if none exists yet, appends it. Used both for the initial
    /// import from the command line and for the `r` hot-resync key.
    ///
    /// EventKit and Jira know nothing about this app's kanban columns, so if
    /// the existing section was already configured as a single board, that
    /// configuration is preserved, along with each card's column (matched
    /// by `external_id`) — unless its completion state changed on the
    /// remote side, in which case it's moved to the first or last column
    /// accordingly. A `boards`-based (multi-board) section, used for Jira,
    /// is always fully rebuilt instead, since its columns come straight
    /// from the live Jira workflow rather than a local, user-set list.
    pub fn upsert_synced_section(sections: &mut Vec<Section>, mut new_section: Section) {
        match sections.iter_mut().find(|s| s.source == new_section.source) {
            Some(existing) => {
                existing.name = new_section.name;
                if let Some(columns) = existing.columns.clone() {
                    let last = columns.len().saturating_sub(1);
                    for todo in new_section.todos.iter_mut() {
                        if let Some(prev) = existing
                            .todos
                            .iter()
                            .find(|t| t.external_id.is_some() && t.external_id == todo.external_id)
                        {
                            todo.column = prev.column;
                        }
                        if todo.is_done && todo.column != last {
                            todo.column = last;
                        } else if !todo.is_done && todo.column == last {
                            todo.column = 0;
                        }
                    }
                    new_section.columns = Some(columns);
                }
                existing.todos = new_section.todos;
                existing.columns = new_section.columns;
                existing.boards = new_section.boards;
            }
            None => sections.push(new_section),
        }
    }

    /// Re-fetches, from EventKit or Jira, the content of every linked
    /// section (Reminders, Calendar, or Jira), overwriting its local
    /// content with the current remote state.
    pub fn sync_linked_sections(&mut self) {
        let reminder_lists: Vec<String> = self
            .sections
            .iter()
            .filter_map(|s| match &s.source {
                SectionSource::Reminders { list } => Some(list.clone()),
                _ => None,
            })
            .collect();
        let calendars: Vec<String> = self
            .sections
            .iter()
            .filter_map(|s| match &s.source {
                SectionSource::Calendar { calendar } => Some(calendar.clone()),
                _ => None,
            })
            .collect();
        let jqls: Vec<String> = self
            .sections
            .iter()
            .filter_map(|s| match &s.source {
                SectionSource::Jira { jql } => Some(jql.clone()),
                _ => None,
            })
            .collect();

        if reminder_lists.is_empty() && calendars.is_empty() && jqls.is_empty() {
            self.message = Some(
                "No sections are linked to Reminders/Calendar/Jira (use --reminders-list, --calendar, or --jira)"
                    .to_string(),
            );
            return;
        }

        let mut errors = Vec::new();

        for list in reminder_lists {
            match crate::eventkit_sync::fetch_reminders_section(&list) {
                Ok(section) => Self::upsert_synced_section(&mut self.sections, section),
                Err(err) => errors.push(format!("Reminders ({list}): {err}")),
            }
        }
        for calendar in calendars {
            match crate::eventkit_sync::fetch_calendar_section(&calendar, 7) {
                Ok(section) => Self::upsert_synced_section(&mut self.sections, section),
                Err(err) => errors.push(format!("Calendar ({calendar}): {err}")),
            }
        }
        for jql in jqls {
            let prefs = crate::config::Config::load().jira;
            match crate::jira_sync::fetch_jira_section(&jql, &prefs) {
                Ok(section) => Self::upsert_synced_section(&mut self.sections, section),
                Err(err) => errors.push(format!("Jira ({jql}): {err}")),
            }
        }

        self.selected_section = self
            .selected_section
            .min(self.sections.len().saturating_sub(1));
        self.reset_todo_cursor();

        self.message = Some(if errors.is_empty() {
            "Synced Reminders/Calendar/Jira".to_string()
        } else {
            format!("Synced with errors: {}", errors.join("; "))
        });
    }

    /// Default path for the task save file: `~/.config/todos/todos.json` —
    /// see `config::app_dir`.
    pub fn default_save_path() -> PathBuf {
        crate::config::app_dir().join("todos.json")
    }

    /// Deserializes sections from JSON. If the content is in the old format
    /// (a flat list of tasks), migrates it by wrapping it in a single
    /// "General" section.
    pub fn parse_sections(content: &str) -> Vec<Section> {
        if let Ok(sections) = serde_json::from_str::<Vec<Section>>(content) {
            return sections;
        }
        match serde_json::from_str::<Vec<Todo>>(content) {
            Ok(todos) if !todos.is_empty() => vec![Section {
                id: Uuid::new_v4(),
                name: "General".to_string(),
                todos,
                source: SectionSource::Local,
                columns: None,
                boards: None,
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

    /// Advances the selection in the currently focused panel.
    pub fn next(&mut self) {
        match self.focus {
            Focus::Sections => self.next_section(),
            Focus::Todos => self.next_todo(),
        }
    }

    /// Moves the selection back in the currently focused panel.
    pub fn previous(&mut self) {
        match self.focus {
            Focus::Sections => self.previous_section(),
            Focus::Todos => self.previous_todo(),
        }
    }

    fn next_section(&mut self) {
        if !self.sections.is_empty() {
            self.selected_section = (self.selected_section + 1) % self.sections.len();
            self.reset_todo_cursor();
        }
    }

    fn previous_section(&mut self) {
        if !self.sections.is_empty() {
            self.selected_section = if self.selected_section == 0 {
                self.sections.len() - 1
            } else {
                self.selected_section - 1
            };
            self.reset_todo_cursor();
        }
    }

    /// Resets the task cursor (plain-list or kanban) to its initial
    /// position — used when switching sections and after a resync.
    fn reset_todo_cursor(&mut self) {
        self.selected_todo = 0;
        self.kanban_board = 0;
        self.kanban_column = 0;
        self.kanban_item = 0;
    }

    fn next_todo(&mut self) {
        let Some(section) = self.sections.get(self.selected_section) else {
            return;
        };
        if section.boards.is_some() {
            self.move_kanban_board(1);
        } else if section.columns.is_some() {
            self.move_kanban_item(1);
        } else if !section.todos.is_empty() {
            self.selected_todo = (self.selected_todo + 1) % section.todos.len();
        }
    }

    fn previous_todo(&mut self) {
        let Some(section) = self.sections.get(self.selected_section) else {
            return;
        };
        if section.boards.is_some() {
            self.move_kanban_board(-1);
        } else if section.columns.is_some() {
            self.move_kanban_item(-1);
        } else if !section.todos.is_empty() {
            self.selected_todo = if self.selected_todo == 0 {
                section.todos.len() - 1
            } else {
                self.selected_todo - 1
            };
        }
    }

    /// Switches to another board (stacked row) within a multi-board
    /// section, with wraparound, keeping the current column (clamped if the
    /// new board has fewer columns).
    fn move_kanban_board(&mut self, delta: i32) {
        let Some(section) = self.sections.get(self.selected_section) else {
            return;
        };
        let Some(boards) = &section.boards else {
            return;
        };
        if boards.is_empty() {
            return;
        }
        let len = boards.len() as i32;
        self.kanban_board = (self.kanban_board as i32 + delta).rem_euclid(len) as usize;
        self.kanban_item = 0;
        let max_col = boards[self.kanban_board].columns.len().saturating_sub(1);
        self.kanban_column = self.kanban_column.min(max_col);
    }

    /// Switches to another card within the current (board, column) cell,
    /// with wraparound. Does nothing if the cell is empty.
    fn move_kanban_item(&mut self, delta: i32) {
        let Some(section) = self.sections.get(self.selected_section) else {
            return;
        };
        let count = section
            .todos
            .iter()
            .filter(|t| t.board_index == self.kanban_board && t.column == self.kanban_column)
            .count();
        if count == 0 {
            return;
        }
        let len = count as i32;
        self.kanban_item = (self.kanban_item as i32 + delta).rem_euclid(len) as usize;
    }

    /// Switches to another column within the current board. Stops at the
    /// edge if there's no column in that direction. Unlike `H`/`L` (which
    /// move a card), this only moves the cursor — it lands on empty
    /// columns too, lazygit-style.
    fn move_kanban_column(&mut self, delta: i32) {
        let Some(section) = self.sections.get(self.selected_section) else {
            return;
        };
        let num_columns = if let Some(boards) = &section.boards {
            boards.get(self.kanban_board).map(|b| b.columns.len())
        } else {
            section.columns.as_ref().map(|c| c.len())
        };
        let Some(num_columns) = num_columns else {
            return;
        };
        if num_columns == 0 {
            return;
        }
        let new_col = self.kanban_column as i32 + delta;
        if new_col < 0 || new_col as usize >= num_columns {
            return;
        }
        self.kanban_column = new_col as usize;
        self.kanban_item = 0;
    }

    /// `h`: on a kanban board with focus on Todos, moves the cursor to the
    /// previous column without touching any card (that's what `H` is for).
    /// In any other case, switches panels — same as Tab/Left arrow.
    pub fn on_left(&mut self) {
        if self.focus == Focus::Todos && self.is_current_section_kanban() {
            self.move_kanban_column(-1);
        } else {
            self.toggle_focus();
        }
    }

    /// Mirrors [`Self::on_left`] but towards the next column.
    pub fn on_right(&mut self) {
        if self.focus == Focus::Todos && self.is_current_section_kanban() {
            self.move_kanban_column(1);
        } else {
            self.toggle_focus();
        }
    }

    fn is_current_section_kanban(&self) -> bool {
        self.selected_section()
            .is_some_and(|s| s.columns.is_some() || s.boards.is_some())
    }

    /// Index, within `section.todos`, of the card under the current kanban
    /// cursor (board + column + position within the cell). `None` if the
    /// section isn't a kanban board, or the cell is empty.
    fn current_kanban_todo_index(&self) -> Option<usize> {
        let section = self.selected_section()?;
        if section.columns.is_none() && section.boards.is_none() {
            return None;
        }
        section
            .todos
            .iter()
            .enumerate()
            .filter(|(_, t)| t.board_index == self.kanban_board && t.column == self.kanban_column)
            .map(|(i, _)| i)
            .nth(self.kanban_item)
    }

    /// Starts editing the selected section's kanban columns. Pre-fills the
    /// input with its current columns (if any) so they can be edited.
    pub fn start_editing_columns(&mut self) {
        let Some(section) = self.selected_section() else {
            self.message = Some("No section is selected".to_string());
            return;
        };
        if section.boards.is_some() {
            self.message = Some(
                "This section already groups several Jira boards; its columns can't be edited by hand"
                    .to_string(),
            );
            return;
        }
        self.input_buffer = section
            .columns
            .as_ref()
            .map(|cols| cols.join(", "))
            .unwrap_or_default();
        self.input_mode = InputMode::AddKanbanColumns;
    }

    /// Applies the columns typed by the user to the selected section. Empty
    /// text reverts the section to a plain list. Existing cards land in the
    /// first column, except those already marked done, which go to the
    /// last one.
    fn apply_kanban_columns(&mut self, text: &str) {
        let Some(section) = self.sections.get_mut(self.selected_section) else {
            return;
        };
        if text.is_empty() {
            section.columns = None;
            self.message = Some(format!("\"{}\" is no longer a kanban board", section.name));
            self.kanban_board = 0;
            self.kanban_column = 0;
            self.kanban_item = 0;
            return;
        }
        let columns: Vec<String> = text
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if columns.is_empty() {
            self.message = Some("Enter at least one column name".to_string());
            return;
        }
        let last = columns.len() - 1;
        for todo in section.todos.iter_mut() {
            todo.column = if todo.is_done { last } else { 0 };
        }
        self.message = Some(format!(
            "\"{}\" is now a kanban board with {} columns",
            section.name,
            columns.len()
        ));
        section.columns = Some(columns);
        self.kanban_board = 0;
        self.kanban_column = 0;
        self.kanban_item = 0;
    }

    /// Moves the card under the current kanban cursor one column to the
    /// left (`delta = -1`) or right (`delta = 1`). If the section is linked
    /// to Reminders or Jira, entering/leaving the last column completes or
    /// un-completes the real item in the corresponding system. The cursor
    /// follows the moved card.
    pub fn move_card(&mut self, delta: i32) {
        if self.focus != Focus::Todos {
            return;
        }
        let Some(section) = self.sections.get(self.selected_section) else {
            return;
        };
        if section.columns.is_none() && section.boards.is_none() {
            self.message =
                Some("This section isn't a kanban board (press K to convert it)".to_string());
            return;
        }
        let Some(todo_idx) = self.current_kanban_todo_index() else {
            self.message = Some("There's no card in this column".to_string());
            return;
        };

        let section = self.sections.get(self.selected_section).unwrap();
        let todo = &section.todos[todo_idx];
        // In a multi-board (Jira) section, the relevant columns are the
        // ones on the card's own board, not on the section itself (which
        // has no columns of its own in that case).
        let columns: Vec<String> = if let Some(boards) = &section.boards {
            let Some(board) = boards.get(todo.board_index) else {
                return;
            };
            board.columns.clone()
        } else {
            section.columns.clone().unwrap_or_default()
        };
        let current_col = todo.column.min(columns.len().saturating_sub(1)) as i32;
        let new_col = current_col + delta;
        if new_col < 0 || new_col as usize >= columns.len() {
            return;
        }
        let new_col = new_col as usize;
        let is_last = new_col == columns.len() - 1;
        let source = section.source.clone();
        let external_id = todo.external_id.clone();
        let column_name = columns[new_col].clone();

        if let Err(err) =
            sync_external_column(&source, external_id.as_deref(), &column_name, is_last)
        {
            self.message = Some(format!("Error syncing: {err}"));
            return;
        }

        if let Some(section) = self.sections.get_mut(self.selected_section)
            && let Some(todo) = section.todos.get_mut(todo_idx)
        {
            todo.column = new_col;
            todo.is_done = is_last;
        }
        self.kanban_column = new_col;
        self.kanban_item = 0;
        self.message = Some(format!("Moved to \"{column_name}\""));
    }

    pub fn toggle_todo(&mut self) {
        if self.focus != Focus::Todos {
            return;
        }
        let Some(section) = self.sections.get(self.selected_section) else {
            return;
        };
        if let SectionSource::Calendar { .. } = &section.source {
            self.message = Some("Calendar events can't be marked done".to_string());
            return;
        }
        if section.columns.is_some() || section.boards.is_some() {
            self.message =
                Some("On a kanban board, move the card with H/L to the last column".to_string());
            return;
        }
        let Some(todo) = section.todos.get(self.selected_todo) else {
            return;
        };
        let new_done = !todo.is_done;
        let source = section.source.clone();
        let external_id = todo.external_id.clone();

        if let Err(err) = sync_external_done(&source, external_id.as_deref(), new_done) {
            self.message = Some(format!("Error syncing: {err}"));
            return;
        }

        if let Some(section) = self.sections.get_mut(self.selected_section)
            && let Some(todo) = section.todos.get_mut(self.selected_todo)
        {
            todo.is_done = new_done;
        }
    }

    /// Starts capturing text to add a section or a task, depending on the
    /// focused panel.
    pub fn start_adding(&mut self) {
        if self.focus == Focus::Todos && self.sections.is_empty() {
            self.message = Some("Create a section first (Tab, then 'a')".to_string());
            return;
        }
        if self.focus == Focus::Todos
            && let Some(section) = self.selected_section()
        {
            match &section.source {
                SectionSource::Calendar { .. } => {
                    self.message = Some("Calendar events can't be created from here".to_string());
                    return;
                }
                SectionSource::Jira { .. } => {
                    self.message = Some("Jira issues can't be created from here".to_string());
                    return;
                }
                SectionSource::Local | SectionSource::Reminders { .. } => {}
            }
        }
        self.input_buffer.clear();
        self.input_mode = match self.focus {
            Focus::Sections => InputMode::AddSection,
            Focus::Todos => InputMode::AddTodo,
        };
    }

    /// Confirms the captured text, creating the corresponding section or task.
    pub fn confirm_input(&mut self) {
        let text = self.input_buffer.trim().to_string();
        match self.input_mode {
            InputMode::AddSection => {
                if !text.is_empty() {
                    self.sections.push(Section::new(text));
                    self.selected_section = self.sections.len() - 1;
                    self.selected_todo = 0;
                }
            }
            InputMode::AddTodo => {
                if !text.is_empty()
                    && let Some(section) = self.sections.get(self.selected_section)
                {
                    let list = match &section.source {
                        SectionSource::Reminders { list } => Some(list.clone()),
                        _ => None,
                    };
                    let mut todo = Todo::new(text.clone());
                    if let Some(list) = list {
                        match crate::eventkit_sync::create_reminder(&list, &text) {
                            Ok(identifier) => todo.external_id = Some(identifier),
                            Err(err) => {
                                self.message = Some(format!("Error creating the reminder: {err}"));
                                self.input_mode = InputMode::Normal;
                                self.input_buffer.clear();
                                return;
                            }
                        }
                    }
                    if let Some(section) = self.sections.get_mut(self.selected_section) {
                        section.todos.push(todo);
                        self.selected_todo = section.todos.len() - 1;
                    }
                }
            }
            InputMode::AddKanbanColumns => self.apply_kanban_columns(&text),
            InputMode::Normal => {}
        }
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    /// Cancels the text capture in progress without creating anything.
    pub fn cancel_input(&mut self) {
        self.input_mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    /// Serializes the sections and writes them to `file_path`, updating
    /// `message` with the result.
    pub fn save_todos(&mut self) {
        let Some(path) = self.file_path.clone() else {
            self.message = Some("No destination file: use `todos <file.json>`".to_string());
            return;
        };

        if let Some(parent) = path.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            self.message = Some(format!("Error creating directory: {err}"));
            return;
        }

        self.message = Some(match serde_json::to_string_pretty(&self.sections) {
            Ok(json) => match fs::write(&path, json) {
                Ok(()) => format!("Saved to {}", path.display()),
                Err(err) => format!("Error saving: {err}"),
            },
            Err(err) => format!("Error serializing: {err}"),
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
    use crate::models::todos::KanbanBoard;

    #[test]
    fn test_save_todos_without_file_path() {
        let mut app = App::new();
        app.save_todos();
        assert!(app.message.unwrap().contains("No destination file"));
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

        assert!(app.message.unwrap().contains("Saved"));

        let content = fs::read_to_string(&file_path).unwrap();
        let saved: Vec<Section> = serde_json::from_str(&content).unwrap();
        assert_eq!(saved.len(), app.sections.len());
        assert!(saved[0].todos[0].is_done);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_parse_sections_migrates_flat_todo_list() {
        let old_format =
            r#"[{"id":"397fabd8-38c5-4b44-a389-b3db35b1d044","text":"something","is_done":false}]"#;
        let sections = App::parse_sections(old_format);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].name, "General");
        assert_eq!(sections[0].todos[0].text, "something");
    }

    #[test]
    fn test_add_section_and_todo() {
        let mut app = App::new();
        app.start_adding();
        app.input_buffer = "Work".to_string();
        app.confirm_input();
        assert_eq!(app.sections.len(), 2);
        assert_eq!(app.sections[1].name, "Work");
        assert_eq!(app.selected_section, 1);

        app.toggle_focus();
        app.start_adding();
        app.input_buffer = "New task".to_string();
        app.confirm_input();
        assert_eq!(app.sections[1].todos.len(), 1);
        assert_eq!(app.sections[1].todos[0].text, "New task");
    }

    #[test]
    fn test_apply_kanban_columns_and_back() {
        let mut app = App::new();
        app.sections[0].todos[0].is_done = true; // already done before converting

        app.start_editing_columns();
        assert_eq!(app.input_mode, InputMode::AddKanbanColumns);
        app.input_buffer = "To do, In progress, Done".to_string();
        app.confirm_input();

        let section = &app.sections[0];
        assert_eq!(
            section.columns,
            Some(vec![
                "To do".to_string(),
                "In progress".to_string(),
                "Done".to_string()
            ])
        );
        // The one already done lands in the last column; the rest in the first.
        assert_eq!(section.todos[0].column, 2);
        assert_eq!(section.todos[1].column, 0);

        // Empty text reverts to a plain list.
        app.start_editing_columns();
        app.input_buffer.clear();
        app.confirm_input();
        assert_eq!(app.sections[0].columns, None);
    }

    #[test]
    fn test_move_card_updates_column_and_done_state() {
        let mut app = App::new();
        app.focus = Focus::Todos;
        app.start_editing_columns();
        app.input_buffer = "To do, In progress, Done".to_string();
        app.confirm_input();

        app.move_card(1);
        assert_eq!(app.sections[0].todos[0].column, 1);
        assert!(!app.sections[0].todos[0].is_done);

        app.move_card(1);
        assert_eq!(app.sections[0].todos[0].column, 2);
        assert!(app.sections[0].todos[0].is_done);

        // At the right edge, moving further does nothing.
        app.move_card(1);
        assert_eq!(app.sections[0].todos[0].column, 2);

        app.move_card(-1);
        assert_eq!(app.sections[0].todos[0].column, 1);
        assert!(!app.sections[0].todos[0].is_done);
    }

    #[test]
    fn test_move_card_without_kanban_shows_message() {
        let mut app = App::new();
        app.focus = Focus::Todos;
        app.move_card(1);
        assert!(app.message.unwrap().contains("isn't a kanban board"));
    }

    #[test]
    fn test_next_todo_cycles_items_within_column() {
        let mut app = App::new();
        app.focus = Focus::Todos;
        app.start_editing_columns();
        app.input_buffer = "A, B".to_string();
        app.confirm_input();
        // The 3 default tasks all start in column 0.
        assert_eq!(app.kanban_item, 0);
        app.next(); // item 0 -> 1
        assert_eq!(app.kanban_item, 1);
        app.next(); // item 1 -> 2
        assert_eq!(app.kanban_item, 2);
        app.next(); // wraps 2 -> 0
        assert_eq!(app.kanban_item, 0);
    }

    #[test]
    fn test_next_todo_moves_between_stacked_boards() {
        let mut app = App::new();
        app.focus = Focus::Todos;
        let section = &mut app.sections[0];
        section.columns = None;
        section.boards = Some(vec![
            KanbanBoard {
                name: "Board A".to_string(),
                columns: vec!["Col0".to_string(), "Col1".to_string()],
            },
            KanbanBoard {
                name: "Board B".to_string(),
                columns: vec!["Col0".to_string(), "Col1".to_string()],
            },
        ]);
        section.todos = vec![
            Todo {
                board_index: 0,
                column: 0,
                ..Todo::new("A0".to_string())
            },
            Todo {
                board_index: 0,
                column: 1,
                ..Todo::new("A1".to_string())
            },
            Todo {
                board_index: 1,
                column: 0,
                ..Todo::new("B0".to_string())
            },
        ];
        assert_eq!(app.kanban_board, 0);
        assert_eq!(app.kanban_column, 0);

        // j should move down to the next board, keeping the column (0).
        app.next();
        assert_eq!(app.kanban_board, 1);
        assert_eq!(app.kanban_column, 0);

        // Wraps back to board 0.
        app.next();
        assert_eq!(app.kanban_board, 0);

        app.previous(); // wraps backwards
        assert_eq!(app.kanban_board, 1);
    }

    #[test]
    fn test_on_left_right_move_between_columns_even_when_empty() {
        let mut app = App::new();
        app.focus = Focus::Todos;
        app.start_editing_columns();
        app.input_buffer = "A, B, C".to_string();
        app.confirm_input();
        // No task was "done", so all 3 land in column A; B and C are
        // empty, but the cursor should still be able to rest there.
        assert_eq!(app.kanban_column, 0);

        app.on_right();
        assert_eq!(app.kanban_column, 1); // B, empty

        app.on_right();
        assert_eq!(app.kanban_column, 2); // C, empty

        app.on_right(); // already at the right edge, nothing happens
        assert_eq!(app.kanban_column, 2);

        app.on_left();
        assert_eq!(app.kanban_column, 1);
    }

    #[test]
    fn test_on_left_toggles_focus_outside_kanban() {
        let mut app = App::new(); // local section, no kanban
        app.focus = Focus::Todos;
        app.on_left();
        assert_eq!(app.focus, Focus::Sections);
    }

    #[test]
    fn test_move_card_uses_current_boards_columns() {
        let mut app = App::new();
        app.focus = Focus::Todos;
        let section = &mut app.sections[0];
        section.columns = None;
        section.boards = Some(vec![
            KanbanBoard {
                name: "Board A".to_string(),
                columns: vec!["Col0".to_string(), "Col1".to_string(), "Col2".to_string()],
            },
            KanbanBoard {
                name: "Board B".to_string(),
                columns: vec!["Solo".to_string()],
            },
        ]);
        section.todos = vec![
            Todo {
                board_index: 0,
                column: 0,
                ..Todo::new("A0".to_string())
            },
            Todo {
                board_index: 1,
                column: 0,
                ..Todo::new("B0".to_string())
            },
        ];

        // Cursor on board B (a single column): moving should do nothing.
        app.kanban_board = 1;
        app.move_card(1);
        assert_eq!(app.sections[0].todos[1].column, 0);

        // Cursor on board A (3 columns): moving does advance.
        app.kanban_board = 0;
        app.move_card(1);
        assert_eq!(app.sections[0].todos[0].column, 1);
        assert_eq!(app.kanban_column, 1); // the cursor follows the moved card
    }
}
