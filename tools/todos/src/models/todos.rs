//! Core data model: a save file is a list of [`Section`]s, each holding a
//! list of [`Todo`]s. A section can be a plain list or a kanban board (one
//! or several stacked boards, for Jira).

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;

/// A single task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: Uuid,
    pub text: String,
    pub is_done: bool,
    /// Identifier of the underlying item in the external system this todo
    /// was imported from (an EventKit reminder identifier, or a Jira issue
    /// key). `None` for todos created locally in the app.
    #[serde(default)]
    pub external_id: Option<String>,
    /// Column index within the relevant column list: `Section::columns` for
    /// a single-board section, or `Section::boards[board_index].columns`
    /// for a multi-board section. Meaningless (and ignored) outside of
    /// kanban sections.
    #[serde(default)]
    pub column: usize,
    /// Index into `Section::boards` identifying which stacked board this
    /// card belongs to. Always `0` for single-board sections.
    #[serde(default)]
    pub board_index: usize,
}

impl Todo {
    pub fn new(text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            text,
            is_done: false,
            external_id: None,
            column: 0,
            board_index: 0,
        }
    }
}

/// Where a section's data comes from: either managed entirely by this app,
/// or synced from an external system (macOS Reminders/Calendar via
/// EventKit, or a Jira Cloud instance).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SectionSource {
    #[default]
    Local,
    Reminders {
        list: String,
    },
    Calendar {
        calendar: String,
    },
    /// Fed by a JQL query against a Jira instance. A single section groups
    /// every project found among the results as its own stacked board — see
    /// [`Section::boards`].
    Jira {
        jql: String,
    },
}

/// One board (typically one Jira project) inside a section that groups
/// several — see [`Section::boards`]. Each card in the section records
/// which board it belongs to via [`Todo::board_index`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanBoard {
    pub name: String,
    pub columns: Vec<String>,
}

/// A named group of todos. Depending on `source`, `columns` and `boards`,
/// it renders as a plain list, a single kanban board, or several kanban
/// boards stacked vertically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: Uuid,
    pub name: String,
    pub todos: Vec<Todo>,
    #[serde(default)]
    pub source: SectionSource,
    /// `Some(names)` turns the section into a single kanban board with
    /// these columns, in order; the last column always represents "done".
    /// `None` keeps the classic plain-list mode. Mutually exclusive with
    /// `boards`.
    #[serde(default)]
    pub columns: Option<Vec<String>>,
    /// `Some(boards)` groups several boards (each with its own columns),
    /// stacked vertically within this one section — used for Jira, when
    /// the fetched issues span multiple projects. Mutually exclusive with
    /// `columns`; takes rendering priority over it when both would apply.
    #[serde(default)]
    pub boards: Option<Vec<KanbanBoard>>,
}

impl Section {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            todos: Vec::new(),
            source: SectionSource::Local,
            columns: None,
            boards: None,
        }
    }
}
