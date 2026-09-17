//! Command-line argument definitions, parsed with `clap`.

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "TUI application for managing TODOs",
    long_about = "TUI application for managing TODOs.\n\n\
        With no arguments, tasks are loaded from and saved to (press `s`) the \
        user's local data file — see the README for its exact path on your OS."
)]
pub struct Cli {
    /// Directory to scan for TODO comments
    #[arg(short, long, value_name = "DIR")]
    pub scan: Option<PathBuf>,

    /// JSON task file to load/save (defaults to the user's data file — see README)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Reminders list to import as a linked section (repeatable)
    #[arg(long = "reminders-list", value_name = "LIST")]
    pub reminders_lists: Vec<String>,

    /// Calendar to import as a linked section (repeatable)
    #[arg(long = "calendar", value_name = "CALENDAR")]
    pub calendars: Vec<String>,

    /// Number of days ahead to import for each --calendar
    #[arg(long = "calendar-days", value_name = "DAYS", default_value_t = 7)]
    pub calendar_days: i64,

    /// Import your assigned, unresolved Jira issues as a "Jira" section
    /// (requires the JIRA_DOMAIN, JIRA_EMAIL and JIRA_API_TOKEN environment variables)
    #[arg(long = "jira")]
    pub jira: bool,

    /// Additional Jira JQL query to import as its own section (repeatable)
    #[arg(long = "jira-jql", value_name = "JQL")]
    pub jira_jql: Vec<String>,
}
