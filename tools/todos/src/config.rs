//! User preferences, stored separately from the data file (`todos.json`)
//! and from credentials (`.env`). Lives at `~/.config/todos/config.json`
//! and is entirely optional: if the file is missing, or its JSON can't be
//! parsed, the app keeps running with defaults (no exclusions, no explicit
//! order) and only prints a warning to stderr.

use serde::Deserialize;
use std::path::PathBuf;

/// Base directory for every file this app persists:
/// `~/.config/todos`. Deliberately `.config` on every platform (rather than
/// each OS's idiomatic location, e.g. `~/Library/Application Support` on
/// macOS) so the same relative layout works for dotfiles synced across
/// machines. The home directory itself is resolved with the `dirs` crate,
/// which uses `$HOME` on macOS/Linux and `%USERPROFILE%` on Windows. Falls
/// back to the current directory if the home directory can't be determined.
pub fn app_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join("todos")
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub jira: JiraConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct JiraConfig {
    /// Column names (Jira status names) to hide whenever they have no
    /// ticket in them. Matched case-insensitively. A column that currently
    /// holds a ticket is never hidden, even if its name is listed here.
    #[serde(default)]
    pub excluded_columns: Vec<String>,
    /// Explicit display order for visible columns (case-insensitive match).
    /// Columns listed here come first, in this order; any other column that
    /// remains visible (not listed here, not excluded) is appended
    /// afterwards, ordered by status category (new → in progress → done)
    /// as a fallback.
    #[serde(default)]
    pub column_order: Vec<String>,
}

impl Config {
    /// Returns `~/.config/todos/config.json` — see [`app_dir`].
    pub fn default_path() -> PathBuf {
        app_dir().join("config.json")
    }

    /// Loads the config from `default_path()`. Never fails: on any problem
    /// (missing file, invalid JSON) it returns `Config::default()`, only
    /// printing a warning to stderr when the file exists but couldn't be
    /// parsed (typically a typo in the user's edits).
    pub fn load() -> Config {
        let path = Self::default_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|err| {
                eprintln!(
                    "Could not parse {}: {err}. Ignoring it and using defaults.",
                    path.display()
                );
                Config::default()
            }),
            Err(_) => Config::default(),
        }
    }
}
