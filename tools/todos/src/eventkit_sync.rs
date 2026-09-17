//! Bridge between the app's model ([`Section`]/[`Todo`]) and EventKit
//! (macOS Reminders and Calendar), via the `eventkit-rs` crate.
//!
//! **macOS only.** `eventkit-rs` itself compiles to an empty shell on other
//! platforms, so this module is split into two implementations behind
//! `#[cfg(target_os = "macos")]`: the real one, and a stub that returns a
//! clear "unsupported" error everywhere else. This lets the rest of the app
//! (and `cargo build`/`cargo test`) work unmodified on Linux and Windows —
//! only the Reminders/Calendar integration itself is unavailable there.
//!
//! Every call here is synchronous and blocks the calling thread while
//! EventKit responds — including the first call ever made, when macOS shows
//! its permission dialog. Since the TUI's main loop is itself synchronous,
//! this is an acceptable trade-off: the app appears to freeze for a moment
//! during a sync, the same way it already does while reading/writing the
//! JSON save file.

use crate::models::todos::{Section, SectionSource, Todo};

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use chrono::{Duration, Local};
    use eventkit::{EventsManager, ReminderDraft, RemindersManager};

    fn reminder_todo_text(item: &eventkit::ReminderItem) -> String {
        match item.due_date {
            Some(due) => format!("{} (due {})", item.title, due.format("%Y-%m-%d %H:%M")),
            None => item.title.clone(),
        }
    }

    fn event_todo_text(item: &eventkit::EventItem) -> String {
        format!(
            "{} — {}",
            item.start_date.format("%Y-%m-%d %H:%M"),
            item.title
        )
    }

    /// Fetches every reminder in `list` and builds the corresponding
    /// read/write-linked [`Section`].
    pub fn fetch_reminders_section(list: &str) -> color_eyre::Result<Section> {
        let manager = RemindersManager::new();
        manager.ensure_authorized()?;

        let items = manager.fetch_reminders(Some(&[list]))?;
        let todos = items
            .into_iter()
            .map(|item| Todo {
                id: uuid::Uuid::new_v4(),
                text: reminder_todo_text(&item),
                is_done: item.completed,
                external_id: Some(item.identifier),
                column: 0,
                board_index: 0,
            })
            .collect();

        Ok(Section {
            id: uuid::Uuid::new_v4(),
            name: list.to_string(),
            todos,
            source: SectionSource::Reminders {
                list: list.to_string(),
            },
            columns: None,
            boards: None,
        })
    }

    /// Fetches every event on `calendar` in the next `days` days.
    pub fn fetch_calendar_section(calendar: &str, days: i64) -> color_eyre::Result<Section> {
        let manager = EventsManager::new();
        manager.ensure_authorized()?;

        let now = Local::now();
        let end = now + Duration::days(days.max(1));
        let items = manager.fetch_events(now, end, Some(&[calendar]))?;
        let todos = items
            .into_iter()
            .map(|item| Todo {
                id: uuid::Uuid::new_v4(),
                text: event_todo_text(&item),
                is_done: false,
                external_id: Some(item.identifier),
                column: 0,
                board_index: 0,
            })
            .collect();

        Ok(Section {
            id: uuid::Uuid::new_v4(),
            name: calendar.to_string(),
            todos,
            source: SectionSource::Calendar {
                calendar: calendar.to_string(),
            },
            columns: None,
            boards: None,
        })
    }

    /// Creates a real reminder in `list` and returns its EventKit identifier.
    pub fn create_reminder(list: &str, title: &str) -> color_eyre::Result<String> {
        let manager = RemindersManager::new();
        manager.ensure_authorized()?;

        let created = manager.create_reminder(&ReminderDraft {
            title,
            calendar_title: Some(list),
            ..Default::default()
        })?;
        Ok(created.identifier)
    }

    /// Marks a reminder completed or not completed in Reminders.app.
    pub fn set_reminder_completed(identifier: &str, completed: bool) -> color_eyre::Result<()> {
        let manager = RemindersManager::new();
        manager.ensure_authorized()?;

        if completed {
            manager.complete_reminder(identifier)?;
        } else {
            manager.uncomplete_reminder(identifier)?;
        }
        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
mod macos {
    use super::*;

    fn unsupported<T>() -> color_eyre::Result<T> {
        Err(color_eyre::eyre::eyre!(
            "Reminders/Calendar integration is only available on macOS"
        ))
    }

    pub fn fetch_reminders_section(_list: &str) -> color_eyre::Result<Section> {
        unsupported()
    }

    pub fn fetch_calendar_section(_calendar: &str, _days: i64) -> color_eyre::Result<Section> {
        unsupported()
    }

    pub fn create_reminder(_list: &str, _title: &str) -> color_eyre::Result<String> {
        unsupported()
    }

    pub fn set_reminder_completed(_identifier: &str, _completed: bool) -> color_eyre::Result<()> {
        unsupported()
    }
}

pub use macos::{
    create_reminder, fetch_calendar_section, fetch_reminders_section, set_reminder_completed,
};
