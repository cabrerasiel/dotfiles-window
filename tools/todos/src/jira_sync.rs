//! Bridge between the app's model ([`Section`]/[`Todo`]) and the Jira Cloud
//! REST API (platform v3 + Agile 1.0). Uses `reqwest` in blocking mode,
//! consistent with the rest of the app (the TUI's main loop is itself
//! synchronous). Cross-platform: this module has no macOS-specific code.
//!
//! Credentials are read from the `JIRA_DOMAIN`, `JIRA_EMAIL` and
//! `JIRA_API_TOKEN` environment variables (never from CLI flags or the task
//! JSON file, so the token never ends up in shell history or on disk in
//! plain form outside of `.env`).
//!
//! Search endpoint: `GET /rest/api/3/search` was retired by Atlassian on
//! May 1st, 2025. This implementation uses its replacement,
//! `POST /rest/api/3/search/jql`, paginating with `nextPageToken` instead
//! of the old `startAt`.
//!
//! Board structure: results are grouped by project into **one** [`Section`]
//! containing one [`KanbanBoard`] per project. Each board's columns are, if
//! possible, **every** status in that project's workflow (fetched from
//! `/rest/api/3/project/{key}/statuses`) — not just the ones your current
//! issues happen to sit in — so empty columns show up too, just like the
//! real board. Columns are ordered per `JiraConfig::column_order` and
//! filtered per `JiraConfig::excluded_columns` (see `status_columns`).
//! Using the Agile board's own column configuration
//! (`/rest/agile/1.0/board/{id}/configuration`) was considered and
//! rejected: in practice it tends to be coarser (Backlog/To Do/In
//! Progress/Done) than the real per-status workflow. The Agile board is
//! still queried once per project, only to use its display name as the
//! board's title — see `find_board_for_project`.

use crate::config::JiraConfig;
use crate::models::todos::{KanbanBoard, Section, SectionSource, Todo};
use serde::Deserialize;
use std::collections::HashMap;

/// Default JQL: "my unfinished issues". Uses `statusCategory != Done`
/// rather than `resolution = Unresolved`: the `resolution` field is often
/// left with a stale value on reopened tickets or workflows that never
/// clear it, while the status category always reflects the issue's actual
/// current state.
pub const DEFAULT_JQL: &str = "assignee = currentUser() AND statusCategory != Done";

struct Credentials {
    domain: String,
    email: String,
    token: String,
}

fn credentials() -> color_eyre::Result<Credentials> {
    let domain = std::env::var("JIRA_DOMAIN").map_err(|_| {
        color_eyre::eyre::eyre!(
            "Missing JIRA_DOMAIN environment variable (e.g. yourcompany.atlassian.net)"
        )
    })?;
    let email = std::env::var("JIRA_EMAIL")
        .map_err(|_| color_eyre::eyre::eyre!("Missing JIRA_EMAIL environment variable"))?;
    let token = std::env::var("JIRA_API_TOKEN").map_err(|_| {
        color_eyre::eyre::eyre!(
            "Missing JIRA_API_TOKEN environment variable (generate one at id.atlassian.com/manage-profile/security/api-tokens)"
        )
    })?;
    Ok(Credentials {
        domain,
        email,
        token,
    })
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    issues: Vec<Issue>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Issue {
    key: String,
    fields: IssueFields,
}

#[derive(Debug, Deserialize)]
struct IssueFields {
    summary: String,
    status: Status,
    project: IssueProject,
}

#[derive(Debug, Deserialize)]
struct IssueProject {
    key: String,
}

#[derive(Debug, Deserialize)]
struct Status {
    id: String,
    name: String,
    #[serde(rename = "statusCategory")]
    status_category: StatusCategory,
}

#[derive(Debug, Deserialize)]
struct StatusCategory {
    key: String,
}

/// Fetches issues for a JQL query, paginating with `nextPageToken` until
/// results are exhausted or a safety cap of 500 issues is reached.
fn search_issues(creds: &Credentials, jql: &str) -> color_eyre::Result<Vec<Issue>> {
    let client = reqwest::blocking::Client::new();
    let url = format!("https://{}/rest/api/3/search/jql", creds.domain);

    let mut all_issues = Vec::new();
    let mut next_page_token: Option<String> = None;

    loop {
        let mut body = serde_json::json!({
            "jql": jql,
            "fields": ["summary", "status", "project"],
            "maxResults": 100,
        });
        if let Some(token) = &next_page_token {
            body["nextPageToken"] = serde_json::Value::String(token.clone());
        }

        let page: SearchResponse = client
            .post(&url)
            .basic_auth(&creds.email, Some(&creds.token))
            .json(&body)
            .send()?
            .error_for_status()?
            .json()?;

        next_page_token = page.next_page_token;
        all_issues.extend(page.issues);

        if next_page_token.is_none() || all_issues.len() >= 500 {
            break;
        }
    }

    Ok(all_issues)
}

#[derive(Debug, Deserialize)]
struct BoardsResponse {
    values: Vec<Board>,
}

#[derive(Debug, Deserialize)]
struct Board {
    #[serde(default)]
    name: String,
    #[serde(rename = "type", default)]
    board_type: String,
}

/// Looks up a project's Agile board, used only to source the section's
/// display name. If several boards exist (e.g. a project with both a Scrum
/// and a Kanban board), prefers the "kanban" one. Not used for columns —
/// see the module-level note on why the board's own column configuration
/// was rejected in favor of the full per-status workflow.
fn find_board_for_project(
    client: &reqwest::blocking::Client,
    creds: &Credentials,
    project_key: &str,
) -> color_eyre::Result<Option<Board>> {
    let url = format!("https://{}/rest/agile/1.0/board", creds.domain);
    let resp: BoardsResponse = client
        .get(&url)
        .query(&[("projectKeyOrId", project_key)])
        .basic_auth(&creds.email, Some(&creds.token))
        .send()?
        .error_for_status()?
        .json()?;

    let mut boards = resp.values;
    if let Some(pos) = boards.iter().position(|b| b.board_type == "kanban") {
        return Ok(Some(boards.swap_remove(pos)));
    }
    Ok(boards.into_iter().next())
}

/// Fetches issues for a JQL query and builds **one** [`Section`] containing
/// one [`KanbanBoard`] per distinct project among the results.
pub fn fetch_jira_section(jql: &str, prefs: &JiraConfig) -> color_eyre::Result<Section> {
    let creds = credentials()?;
    let client = reqwest::blocking::Client::new();
    let issues = search_issues(&creds, jql)?;

    let mut project_order: Vec<String> = Vec::new();
    let mut by_project: HashMap<String, Vec<Issue>> = HashMap::new();
    for issue in issues {
        let key = issue.fields.project.key.clone();
        if !by_project.contains_key(&key) {
            project_order.push(key.clone());
        }
        by_project.entry(key).or_default().push(issue);
    }

    let mut boards = Vec::new();
    let mut todos = Vec::new();
    for (board_index, project_key) in project_order.into_iter().enumerate() {
        let project_issues = by_project.remove(&project_key).unwrap_or_default();
        let (board, board_todos) = build_project_board(
            &client,
            &creds,
            &project_key,
            project_issues,
            prefs,
            board_index,
        );
        boards.push(board);
        todos.extend(board_todos);
    }

    let name = if jql == DEFAULT_JQL {
        "Jira".to_string()
    } else {
        format!("Jira: {jql}")
    };

    Ok(Section {
        id: uuid::Uuid::new_v4(),
        name,
        todos,
        source: SectionSource::Jira {
            jql: jql.to_string(),
        },
        columns: None,
        boards: Some(boards),
    })
}

#[derive(Debug, Deserialize)]
struct IssueTypeStatuses {
    statuses: Vec<ProjectStatus>,
}

#[derive(Debug, Deserialize)]
struct ProjectStatus {
    id: String,
    name: String,
    #[serde(rename = "statusCategory")]
    status_category: StatusCategory,
}

/// Fetches every possible status in a project's workflow (the union across
/// all issue types), so the board can show empty columns too, not only the
/// ones that currently hold a ticket.
fn fetch_project_statuses(
    client: &reqwest::blocking::Client,
    creds: &Credentials,
    project_key: &str,
) -> color_eyre::Result<Vec<ProjectStatus>> {
    let url = format!(
        "https://{}/rest/api/3/project/{project_key}/statuses",
        creds.domain
    );
    let issue_types: Vec<IssueTypeStatuses> = client
        .get(&url)
        .basic_auth(&creds.email, Some(&creds.token))
        .send()?
        .error_for_status()?
        .json()?;

    let mut seen = std::collections::HashSet::new();
    let mut statuses = Vec::new();
    for issue_type in issue_types {
        for status in issue_type.statuses {
            if seen.insert(status.id.clone()) {
                statuses.push(status);
            }
        }
    }
    Ok(statuses)
}

/// Builds one project's board: columns are, when possible, **every** status
/// in that project's workflow (empty columns included); if that lookup
/// fails, falls back to just the statuses present among the fetched
/// issues. `board_index` is this board's position in `Section::boards`,
/// stamped onto every returned todo.
fn build_project_board(
    client: &reqwest::blocking::Client,
    creds: &Credentials,
    project_key: &str,
    issues: Vec<Issue>,
    prefs: &JiraConfig,
    board_index: usize,
) -> (KanbanBoard, Vec<Todo>) {
    let name = find_board_for_project(client, creds, project_key)
        .ok()
        .flatten()
        .map(|b| b.name)
        .filter(|n| !n.is_empty())
        .unwrap_or_else(|| format!("Jira: {project_key}"));

    let all_statuses = fetch_project_statuses(client, creds, project_key).ok();
    let (column_names, column_indices) = status_columns(all_statuses.as_deref(), &issues, prefs);

    let todos = issues
        .into_iter()
        .zip(column_indices)
        .map(|(issue, column)| Todo {
            id: uuid::Uuid::new_v4(),
            text: format!("{} — {}", issue.key, issue.fields.summary),
            is_done: issue.fields.status.status_category.key == "done",
            external_id: Some(issue.key),
            column,
            board_index,
        })
        .collect();

    (
        KanbanBoard {
            name,
            columns: column_names,
        },
        todos,
    )
}

/// Relative ordering of a Jira status category: new < in progress < done.
fn category_rank(key: &str) -> u8 {
    match key {
        "new" => 0,
        "done" => 2,
        _ => 1, // "indeterminate" and any unknown category
    }
}

/// Builds the final column list and, for each issue, its column index.
///
/// Columns named in `prefs.column_order` come first, in that order;
/// everything else is appended afterwards, ordered by status category (new
/// → in progress → done) as a fallback. The base set of columns is either
/// `all_statuses` (the project's full workflow, including currently-empty
/// statuses) when available, or just the statuses seen among `issues`
/// otherwise. Either way, if an issue's status wasn't in the base set (a
/// workflow that changed after the statuses were cached), it still gets its
/// own column so the card is never dropped. Finally, columns listed in
/// `prefs.excluded_columns` (case-insensitive) are dropped if — and only
/// if — they ended up empty; a column holding a ticket is never hidden.
///
/// Returns the final column names and, in the same order as `issues`, each
/// issue's column index.
fn status_columns(
    all_statuses: Option<&[ProjectStatus]>,
    issues: &[Issue],
    prefs: &JiraConfig,
) -> (Vec<String>, Vec<usize>) {
    let mut ids: Vec<String> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    let mut categories: Vec<String> = Vec::new();

    if let Some(all) = all_statuses {
        for status in all {
            ids.push(status.id.clone());
            names.push(status.name.clone());
            categories.push(status.status_category.key.clone());
        }
    }
    for issue in issues {
        if !ids.contains(&issue.fields.status.id) {
            ids.push(issue.fields.status.id.clone());
            names.push(issue.fields.status.name.clone());
            categories.push(issue.fields.status.status_category.key.clone());
        }
    }

    let order_lower: Vec<String> = prefs
        .column_order
        .iter()
        .map(|s| s.to_lowercase())
        .collect();
    let mut order: Vec<usize> = (0..names.len()).collect();
    order.sort_by_key(|&i| {
        match order_lower
            .iter()
            .position(|o| *o == names[i].to_lowercase())
        {
            Some(pos) => (0u8, pos),
            None => (1u8, category_rank(&categories[i]) as usize),
        }
    });

    let mut final_pos = vec![0usize; names.len()];
    for (final_idx, &orig_idx) in order.iter().enumerate() {
        final_pos[orig_idx] = final_idx;
    }

    let sorted_names: Vec<String> = order.into_iter().map(|i| names[i].clone()).collect();

    let id_to_slot: HashMap<&str, usize> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), i))
        .collect();
    let raw_indices: Vec<usize> = issues
        .iter()
        .map(|issue| final_pos[id_to_slot[issue.fields.status.id.as_str()]])
        .collect();

    let occupied: std::collections::HashSet<usize> = raw_indices.iter().copied().collect();
    let excluded_lower: std::collections::HashSet<String> = prefs
        .excluded_columns
        .iter()
        .map(|s| s.to_lowercase())
        .collect();
    let keep: Vec<usize> = (0..sorted_names.len())
        .filter(|&pos| {
            occupied.contains(&pos) || !excluded_lower.contains(&sorted_names[pos].to_lowercase())
        })
        .collect();

    let mut remap = vec![0usize; sorted_names.len()];
    for (new_idx, &old_idx) in keep.iter().enumerate() {
        remap[old_idx] = new_idx;
    }

    let final_names = keep.iter().map(|&pos| sorted_names[pos].clone()).collect();
    let final_indices = raw_indices.iter().map(|&pos| remap[pos]).collect();

    (final_names, final_indices)
}

#[derive(Debug, Deserialize)]
struct TransitionsResponse {
    transitions: Vec<Transition>,
}

#[derive(Debug, Deserialize)]
struct Transition {
    id: String,
    to: TransitionTarget,
}

#[derive(Debug, Deserialize)]
struct TransitionTarget {
    name: String,
}

/// Attempts to transition the issue to the status whose **name** matches
/// `target_status_name` exactly (case-insensitive) — the same name shown as
/// the column in the app. Jira only exposes the transitions legal from the
/// issue's current status; if none of them leads to that exact name, this
/// returns an error instead of guessing an approximate transition (which
/// would otherwise move a card to one column visually while the real issue
/// ends up in a completely different status).
pub fn set_issue_status(issue_key: &str, target_status_name: &str) -> color_eyre::Result<()> {
    let creds = credentials()?;
    let client = reqwest::blocking::Client::new();
    let transitions_url = format!(
        "https://{}/rest/api/3/issue/{issue_key}/transitions",
        creds.domain
    );

    let transitions: TransitionsResponse = client
        .get(&transitions_url)
        .basic_auth(&creds.email, Some(&creds.token))
        .send()?
        .error_for_status()?
        .json()?;

    let target = transitions
        .transitions
        .iter()
        .find(|t| t.to.name.eq_ignore_ascii_case(target_status_name))
        .ok_or_else(|| {
            color_eyre::eyre::eyre!(
                "{issue_key} has no direct transition available to \"{target_status_name}\" from its current status"
            )
        })?;

    client
        .post(&transitions_url)
        .basic_auth(&creds.email, Some(&creds.token))
        .json(&serde_json::json!({ "transition": { "id": target.id } }))
        .send()?
        .error_for_status()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(key: &str, status_id: &str, status_name: &str, category: &str) -> Issue {
        Issue {
            key: key.to_string(),
            fields: IssueFields {
                summary: "summary".to_string(),
                status: Status {
                    id: status_id.to_string(),
                    name: status_name.to_string(),
                    status_category: StatusCategory {
                        key: category.to_string(),
                    },
                },
                project: IssueProject {
                    key: "PROJ".to_string(),
                },
            },
        }
    }

    fn project_status(id: &str, name: &str, category: &str) -> ProjectStatus {
        ProjectStatus {
            id: id.to_string(),
            name: name.to_string(),
            status_category: StatusCategory {
                key: category.to_string(),
            },
        }
    }

    fn prefs(excluded: &[&str], order: &[&str]) -> JiraConfig {
        JiraConfig {
            excluded_columns: excluded.iter().map(|s| s.to_string()).collect(),
            column_order: order.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_status_columns_falls_back_to_category_order_without_column_order() {
        let issues = vec![
            issue("A-1", "3", "Done", "done"),
            issue("A-2", "2", "In Progress", "indeterminate"),
            issue("A-3", "1", "To Do", "new"),
        ];
        let (names, indices) = status_columns(None, &issues, &prefs(&[], &[]));

        assert_eq!(names, vec!["To Do", "In Progress", "Done"]);
        // A-1 (Done) -> col 2, A-2 (In Progress) -> col 1, A-3 (To Do) -> col 0
        assert_eq!(indices, vec![2, 1, 0]);
    }

    #[test]
    fn test_status_columns_explicit_order_overrides_category_order() {
        let issues = vec![
            issue("A-1", "3", "Done", "done"),
            issue("A-2", "2", "Code Review", "indeterminate"),
            issue("A-3", "1", "To Do", "new"),
        ];
        // Deliberately not the natural new -> in progress -> done order.
        let (names, indices) = status_columns(
            None,
            &issues,
            &prefs(&[], &["Done", "To Do", "Code Review"]),
        );

        assert_eq!(names, vec!["Done", "To Do", "Code Review"]);
        assert_eq!(indices, vec![0, 2, 1]);
    }

    #[test]
    fn test_status_columns_order_matching_is_case_insensitive() {
        let issues = vec![issue("A-1", "1", "Done", "done")];
        let (names, _) = status_columns(None, &issues, &prefs(&[], &["done"]));
        assert_eq!(names, vec!["Done"]);
    }

    #[test]
    fn test_status_columns_hides_excluded_columns_only_when_empty() {
        let all_statuses = vec![
            project_status("1", "Backlog", "new"),
            project_status("2", "To Do", "new"),
            project_status("3", "In Progress", "indeterminate"),
            project_status("4", "Done", "done"),
        ];
        let issues = vec![issue("A-1", "2", "To Do", "new")];
        // All three are "excluded", but only the ones with no ticket should
        // actually disappear.
        let (names, indices) = status_columns(
            Some(&all_statuses),
            &issues,
            &prefs(&["Backlog", "To Do", "Done"], &[]),
        );

        assert_eq!(names, vec!["To Do", "In Progress"]);
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn test_status_columns_exclusion_is_case_insensitive() {
        let all_statuses = vec![
            project_status("1", "To Do", "new"),
            project_status("2", "Done", "done"),
        ];
        let issues = vec![issue("A-1", "2", "Done", "done")];
        let (names, _) = status_columns(Some(&all_statuses), &issues, &prefs(&["TO DO"], &[]));

        assert_eq!(names, vec!["Done"]);
    }

    #[test]
    fn test_status_columns_keeps_issue_status_missing_from_workflow_snapshot() {
        // The project's cached workflow only knows about "To Do", but an
        // issue comes back with a status that isn't in that snapshot
        // (e.g. a status added to the workflow after it was fetched).
        let all_statuses = vec![project_status("1", "To Do", "new")];
        let issues = vec![issue("A-1", "999", "Mystery", "indeterminate")];
        let (names, indices) = status_columns(Some(&all_statuses), &issues, &prefs(&[], &[]));

        assert_eq!(names, vec!["To Do", "Mystery"]);
        assert_eq!(indices, vec![1]);
    }

    /// Live integration test against a real Jira instance. Requires
    /// JIRA_DOMAIN/JIRA_EMAIL/JIRA_API_TOKEN (via `.env` or exported) and is
    /// therefore marked `#[ignore]` so plain `cargo test` never hits the
    /// network. Run it with:
    /// `cargo test jira_sync::tests::test_live_fetch_default_jql -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn test_live_fetch_default_jql() {
        let _ = dotenvy::dotenv();
        let prefs = crate::config::Config::load().jira;
        let section = fetch_jira_section(DEFAULT_JQL, &prefs).expect("fetch_jira_section failed");
        println!("Section \"{}\"", section.name);
        let boards = section.boards.as_ref().expect("expected boards");
        for (board_idx, board) in boards.iter().enumerate() {
            println!("  Board \"{}\" — columns: {:?}", board.name, board.columns);
            for todo in section.todos.iter().filter(|t| t.board_index == board_idx) {
                println!(
                    "    [col {}] {} (external_id={:?})",
                    todo.column, todo.text, todo.external_id
                );
            }
        }
    }
}
