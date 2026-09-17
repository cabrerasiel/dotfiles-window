pub mod app;
pub mod cli;
pub mod config;
pub mod eventkit_sync;
pub mod handler;
pub mod jira_sync;
pub mod models;
pub mod scanner;
pub mod ui;

use app::App;
use clap::Parser;
use cli::Cli;
use models::todos::{Section, SectionSource};
use scanner::scan_directory;
use std::fs;
use uuid::Uuid;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    // Loads variables from a local .env file (JIRA_DOMAIN, JIRA_EMAIL,
    // JIRA_API_TOKEN) if one exists. Doesn't fail if it's missing: those
    // variables can be set some other way (export, a launcher script, a
    // process manager, etc.).
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    // Without --scan, tasks default to the user's data file (see
    // `App::default_save_path`) unless an explicit --file is given. In
    // --scan mode, only save if --file was passed, so a scan's results
    // never overwrite the user's local task database.
    let save_path = if cli.scan.is_some() {
        cli.file.clone()
    } else {
        Some(cli.file.clone().unwrap_or_else(App::default_save_path))
    };

    let mut sections = if let Some(scan_dir) = &cli.scan {
        let todos = scan_directory(scan_dir);
        if todos.is_empty() {
            Vec::new()
        } else {
            vec![Section {
                id: Uuid::new_v4(),
                name: format!("Scan: {}", scan_dir.display()),
                todos,
                source: SectionSource::Local,
                columns: None,
                boards: None,
            }]
        }
    } else if let Some(file_path) = &cli.file {
        if file_path.exists() {
            let content = fs::read_to_string(file_path)?;
            App::parse_sections(&content)
        } else {
            Vec::new()
        }
    } else if save_path.as_ref().is_some_and(|p| p.exists()) {
        let content = fs::read_to_string(save_path.as_ref().unwrap())?;
        App::parse_sections(&content)
    } else {
        App::default_sections()
    };

    for list in &cli.reminders_lists {
        match eventkit_sync::fetch_reminders_section(list) {
            Ok(section) => App::upsert_synced_section(&mut sections, section),
            Err(err) => eprintln!("Could not import Reminders list '{list}': {err}"),
        }
    }
    for calendar in &cli.calendars {
        match eventkit_sync::fetch_calendar_section(calendar, cli.calendar_days) {
            Ok(section) => App::upsert_synced_section(&mut sections, section),
            Err(err) => eprintln!("Could not import calendar '{calendar}': {err}"),
        }
    }
    if cli.jira || !cli.jira_jql.is_empty() {
        let prefs = &config::Config::load().jira;
        if cli.jira {
            match jira_sync::fetch_jira_section(jira_sync::DEFAULT_JQL, prefs) {
                Ok(section) => App::upsert_synced_section(&mut sections, section),
                Err(err) => eprintln!("Could not import Jira: {err}"),
            }
        }
        for jql in &cli.jira_jql {
            match jira_sync::fetch_jira_section(jql, prefs) {
                Ok(section) => App::upsert_synced_section(&mut sections, section),
                Err(err) => eprintln!("Could not import Jira query '{jql}': {err}"),
            }
        }
    }

    let mut app = App::with_sections(sections);
    app.file_path = save_path;
    let terminal = ratatui::init();
    let result = app.run(terminal);
    ratatui::restore();
    result
}
