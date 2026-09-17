# todos

A terminal (TUI) task manager built with [Ratatui](https://ratatui.rs). Beyond
a plain local todo list, it can turn any section into a kanban board, and it
can sync sections from macOS Reminders/Calendar and from Jira Cloud.

## Features

- **Local sections and tasks** — organize todos into named sections, with a
  JSON save file you can inspect, back up, or version-control yourself.
- **Kanban boards** — turn any section into a kanban board with custom
  columns, and move cards between them with the keyboard.
- **Directory scanning** — pull `TODO` comments out of a codebase into a
  read-only section (`--scan`).
- **macOS Reminders & Calendar** *(macOS only)* — import a Reminders list or
  a Calendar as a linked section; completing a card marks the real reminder
  done, and adding a task in a linked section creates a real reminder.
- **Jira Cloud** *(cross-platform)* — import your assigned issues as a
  kanban section, one board per project, with columns matching that
  project's real workflow statuses. Moving a card transitions the real
  Jira issue.

## Platform support

| Feature                              | macOS | Linux | Windows |
| ------------------------------------- | :---: | :---: | :-----: |
| Local todos, sections, kanban boards  |  ✅   |  ✅   |   ✅    |
| Directory scan (`--scan`)             |  ✅   |  ✅   |   ✅    |
| Jira sync                             |  ✅   |  ✅   |   ✅    |
| Reminders / Calendar sync             |  ✅   |  ❌   |   ❌    |

Reminders/Calendar integration depends on Apple's EventKit framework and can
only run on macOS. On other platforms, `--reminders-list` / `--calendar` fail
with a clear "only available on macOS" error instead of the app refusing to
build — everything else works the same everywhere.

The user data and config file paths below are resolved with the [`dirs`
crate](https://docs.rs/dirs), which uses `$HOME` on macOS/Linux and
`%USERPROFILE%` (via the Windows Known Folder API) on Windows — no manual
path setup is required on any platform.

## Building

```bash
cargo build --release
```

The binary is at `target/release/todos`. There's nothing platform-specific
to install beyond a Rust toolchain (`rustup`) — Reminders/Calendar support
is compiled in automatically on macOS and compiled out (as a stub) on other
platforms.

## Where your data lives

Everything lives under `~/.config/todos/` — deliberately `.config` on every
platform (rather than each OS's own idiomatic location) so the layout stays
consistent for dotfiles synced across machines.

| File                              | macOS / Linux                        | Windows                                    | Purpose |
| ---------------------------------- | ------------------------------------- | -------------------------------------------- | ------- |
| `~/.config/todos/todos.json`       | `/Users/you/.config/todos/todos.json` | `C:\Users\you\.config\todos\todos.json`      | Default save file: your sections and tasks. |
| `~/.config/todos/config.json`      | `/Users/you/.config/todos/config.json` | `C:\Users\you\.config\todos\config.json`    | Optional preferences (currently: Jira column filtering/ordering). |
| `.env`                             | next to the binary / in the working directory | same | Jira credentials. Never committed — see below. |

The `~/.config/todos/` directory is created automatically the first time you
save (press `s`) — no manual setup needed. If `config.json` doesn't exist,
or contains invalid JSON, the app just uses defaults and prints a warning to
stderr — it never crashes on a bad config.

You can also point the app at a specific file instead of the default:

```bash
todos path/to/my-tasks.json
```

## Setting up Jira sync

1. Copy the example env file and edit it:

   ```bash
   cp .env.example .env
   ```

2. Generate a personal API token at
   <https://id.atlassian.com/manage-profile/security/api-tokens> ("Create
   API token"), then fill in `.env`:

   ```env
   JIRA_DOMAIN=yourcompany.atlassian.net
   JIRA_EMAIL=you@example.com
   JIRA_API_TOKEN=the-token-you-just-generated
   ```

   `.env` is listed in `.gitignore` (`.env`, `.env.*`, with `.env.example`
   explicitly un-ignored) — it never gets committed. Treat the API token
   like a password: if it's ever pasted somewhere outside your own `.env`
   (a chat, a ticket, a log), revoke it and generate a new one.

3. Run with Jira import enabled:

   ```bash
   todos --jira
   ```

`.env` is loaded automatically on startup from the current working
directory (and its parents) if present — no need to `export` the variables
yourself, though that also works.

### Jira column preferences

`~/.config/todos/config.json` lets you hide noisy empty columns and pin a
display order, shared across every Jira board:

```json
{
  "jira": {
    "excluded_columns": ["On Hold", "To Do", "Ready for QA", "Cancelled"],
    "column_order": [
      "In Development",
      "Code Review",
      "In QA",
      "In UAT",
      "Done"
    ]
  }
}
```

- `excluded_columns` hides a column **only** while it's empty. A column
  holding a real ticket is never hidden, no matter what's in this list.
- `column_order` pins the listed columns first, in that exact order; any
  other visible column is appended afterwards, grouped by status category
  (new → in progress → done).
- Matching is case-insensitive; names must otherwise match the Jira status
  name exactly.
- Re-read on every resync (press `r`) — no restart needed after editing it.

## Command-line usage

```bash
todos [FILE]                                  # load/save a specific file
todos --scan <DIR>                            # read-only section of TODO comments found in DIR
todos --reminders-list "Groceries"             # macOS only, repeatable
todos --calendar "Work" --calendar-days 14     # macOS only, repeatable
todos --jira                                   # your assigned, unresolved Jira issues
todos --jira-jql "project = ABC AND sprint in openSprints()"   # custom JQL, repeatable
```

Run `todos --help` for the full flag reference.

## Keybindings

| Key       | Action |
| --------- | ------ |
| `Tab`, `←`/`→` | Switch focus between the Sections and Todos panels |
| `j`/`k`, `↓`/`↑` | Move the selection down/up (within a column; between stacked boards once you reach the top/bottom, on a multi-board section) |
| `h`/`l`   | Switch panels — **or**, inside a kanban board with focus on Todos, move the cursor to the previous/next column (lands on empty columns too, without touching any card) |
| `H`/`L`   | Move the selected card to the previous/next column. Crossing into/out of the last column completes/un-completes the linked Reminder or Jira issue; on Jira, any move transitions the real issue to the exact matching status name |
| `Space`   | Toggle a task done/not-done (plain-list sections only — kanban sections use `H`/`L` instead) |
| `a`       | Add a new section (Sections panel) or task (Todos panel) |
| `K`       | Edit the selected section's kanban columns (comma-separated; empty input reverts it to a plain list) — not available on a Jira multi-board section |
| `s`       | Save to the current file |
| `r`       | Re-sync every linked section (Reminders, Calendar, Jira) from its source |
| `q`, `Esc`, `Ctrl+C` | Quit (`Esc` cancels text input instead, if you're typing) |
| `Enter`   | Confirm text input |

## Known limitations

- Reminders/Calendar sections can't be resynced without losing manual
  reordering beyond done/not-done — see the doc comment on
  `App::upsert_synced_section` for exactly what's preserved.
- Moving a Jira card requires a transition that leads directly to a status
  with that exact name from the issue's current status. If Jira's workflow
  doesn't expose one (e.g. no direct path from "In Development" to "Done"),
  the move is rejected with an error rather than guessing an approximate
  transition.
- Creating new todos is disabled for Calendar and Jira sections (there's no
  sensible one-line way to create a calendar event or a full Jira issue).
