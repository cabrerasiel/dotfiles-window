//! Renders the terminal UI: a lazygit/lazydocker-style panel layout —
//! sections on the left, the active section's tasks on the right, and a
//! status/help bar at the bottom.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, Focus, InputMode};
use crate::models::todos::SectionSource;

const ACTIVE_BORDER: Color = Color::Cyan;
const INACTIVE_BORDER: Color = Color::DarkGray;

pub fn render(app: &App, frame: &mut Frame) {
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(4)])
        .split(frame.area());

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(outer[0]);

    render_sections(app, frame, columns[0]);
    render_todos(app, frame, columns[1]);
    render_status_bar(app, frame, outer[1]);
}

fn render_sections(app: &App, frame: &mut Frame, area: Rect) {
    let focused = app.focus == Focus::Sections;

    let items: Vec<ListItem> = app
        .sections
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let selected = i == app.selected_section;
            let done = section.todos.iter().filter(|t| t.is_done).count();
            let source_tag = match section.source {
                SectionSource::Local => "",
                SectionSource::Reminders { .. } => "[Reminders] ",
                SectionSource::Calendar { .. } => "[Calendar] ",
                SectionSource::Jira { .. } => "[Jira] ",
            };
            let kanban_tag = if section.columns.is_some() || section.boards.is_some() {
                "[Kanban] "
            } else {
                ""
            };
            let text = format!(
                "{source_tag}{kanban_tag}{} ({done}/{})",
                section.name,
                section.todos.len()
            );
            ListItem::new(Span::styled(text, row_style(selected, focused)))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(focused))
        .title(" Sections ");
    frame.render_widget(List::new(items).block(block), area);
}

fn render_todos(app: &App, frame: &mut Frame, area: Rect) {
    let focused = app.focus == Focus::Todos;

    match app.selected_section() {
        Some(section) if section.boards.is_some() => {
            render_stacked_boards(app, section, frame, area, focused)
        }
        Some(section) if section.columns.is_some() => {
            render_kanban_board(app, section, frame, area, focused)
        }
        Some(section) => render_todo_list(app, section, frame, area, focused),
        None => {
            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(border_style(focused))
                .title(" Todos ");
            frame.render_widget(block, area);
        }
    }
}

fn render_todo_list(
    app: &App,
    section: &crate::models::todos::Section,
    frame: &mut Frame,
    area: Rect,
    focused: bool,
) {
    let items: Vec<ListItem> = section
        .todos
        .iter()
        .enumerate()
        .map(|(i, todo)| {
            let selected = i == app.selected_todo;
            let status = if todo.is_done { "[✓] " } else { "[ ] " };
            let text = format!("{status}{}", todo.text);
            ListItem::new(Span::styled(text, row_style(selected, focused)))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(focused))
        .title(format!(" Todos — {} ", section.name));
    frame.render_widget(List::new(items).block(block), area);
}

fn render_kanban_board(
    app: &App,
    section: &crate::models::todos::Section,
    frame: &mut Frame,
    area: Rect,
    focused: bool,
) {
    let columns = section.columns.as_ref().expect("kanban section");
    let constraints: Vec<Constraint> = columns
        .iter()
        .map(|_| Constraint::Ratio(1, columns.len() as u32))
        .collect();
    let areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    for (col_idx, col_name) in columns.iter().enumerate() {
        let is_cursor_here = focused && app.kanban_column == col_idx;
        let items: Vec<ListItem> = section
            .todos
            .iter()
            .filter(|todo| todo.column == col_idx)
            .enumerate()
            .map(|(item_idx, todo)| {
                let selected = is_cursor_here && app.kanban_item == item_idx;
                ListItem::new(Span::styled(
                    todo.text.clone(),
                    row_style(selected, focused),
                ))
            })
            .collect();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(cursor_border_style(is_cursor_here, focused))
            .title(format!(" {col_name} "));
        frame.render_widget(List::new(items).block(block), areas[col_idx]);
    }
}

/// Renders a multi-board section (e.g. Jira with several projects): one
/// full board per row, stacked vertically, each with a header showing its
/// name and its own columns below.
fn render_stacked_boards(
    app: &App,
    section: &crate::models::todos::Section,
    frame: &mut Frame,
    area: Rect,
    focused: bool,
) {
    let boards = section.boards.as_ref().expect("multi-board section");
    if boards.is_empty() {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style(focused))
            .title(format!(" {} ", section.name));
        frame.render_widget(block, area);
        return;
    }

    let row_constraints: Vec<Constraint> = boards
        .iter()
        .map(|_| Constraint::Ratio(1, boards.len() as u32))
        .collect();
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    for (board_idx, board) in boards.iter().enumerate() {
        let row = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(1)])
            .split(rows[board_idx]);

        let is_cursor_board = focused && app.kanban_board == board_idx;
        let header_style = if is_cursor_board {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        };
        let header = Paragraph::new(Span::styled(board.name.clone(), header_style));
        frame.render_widget(header, row[0]);

        if board.columns.is_empty() {
            continue;
        }
        let col_constraints: Vec<Constraint> = board
            .columns
            .iter()
            .map(|_| Constraint::Ratio(1, board.columns.len() as u32))
            .collect();
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(row[1]);

        for (col_idx, col_name) in board.columns.iter().enumerate() {
            let is_cursor_here = is_cursor_board && app.kanban_column == col_idx;
            let items: Vec<ListItem> = section
                .todos
                .iter()
                .filter(|todo| todo.board_index == board_idx && todo.column == col_idx)
                .enumerate()
                .map(|(item_idx, todo)| {
                    let selected = is_cursor_here && app.kanban_item == item_idx;
                    ListItem::new(Span::styled(
                        todo.text.clone(),
                        row_style(selected, focused),
                    ))
                })
                .collect();

            let block = Block::default()
                .borders(Borders::ALL)
                .border_style(cursor_border_style(is_cursor_here, focused))
                .title(format!(" {col_name} "));
            frame.render_widget(List::new(items).block(block), cols[col_idx]);
        }
    }
}

fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(ACTIVE_BORDER)
    } else {
        Style::default().fg(INACTIVE_BORDER)
    }
}

/// Border of a kanban column: highlighted in yellow when the cursor rests
/// there (even if empty, lazygit-style), the normal border otherwise.
fn cursor_border_style(is_cursor_here: bool, focused: bool) -> Style {
    if is_cursor_here {
        Style::default().fg(Color::Yellow)
    } else {
        border_style(focused)
    }
}

fn row_style(selected: bool, focused: bool) -> Style {
    match (selected, focused) {
        (true, true) => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        (true, false) => Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::BOLD),
        (false, _) => Style::default(),
    }
}

fn render_status_bar(app: &App, frame: &mut Frame, area: Rect) {
    match app.input_mode {
        InputMode::AddSection => {
            frame.render_widget(input_paragraph("New section", &app.input_buffer), area);
        }
        InputMode::AddTodo => {
            frame.render_widget(input_paragraph("New task", &app.input_buffer), area);
        }
        InputMode::AddKanbanColumns => {
            frame.render_widget(
                input_paragraph(
                    "Comma-separated columns (empty = remove kanban)",
                    &app.input_buffer,
                ),
                area,
            );
        }
        InputMode::Normal => frame.render_widget(help_paragraph(app), area),
    }
}

fn input_paragraph<'a>(label: &'a str, buffer: &'a str) -> Paragraph<'a> {
    let line = Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(buffer),
        Span::styled("▎", Style::default().fg(Color::Yellow)),
    ]);
    Paragraph::new(line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" Enter: confirm · Esc: cancel "),
    )
}

fn help_paragraph(app: &App) -> Paragraph<'static> {
    let key_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let keys = Line::from(vec![
        Span::styled("Tab", key_style),
        Span::raw(" panel  "),
        Span::styled("j/k", key_style),
        Span::raw(" move  "),
        Span::styled("h/l", key_style),
        Span::raw(" panel / column (kanban)  "),
        Span::styled("a", key_style),
        Span::raw(" add  "),
        Span::styled("space", key_style),
        Span::raw(" complete  "),
        Span::styled("s", key_style),
        Span::raw(" save  "),
        Span::styled("r", key_style),
        Span::raw(" sync reminders/calendar/jira  "),
        Span::styled("K", key_style),
        Span::raw(" kanban columns  "),
        Span::styled("H/L", key_style),
        Span::raw(" move card  "),
        Span::styled("q", key_style),
        Span::raw(" quit"),
    ]);

    let mut lines = vec![keys];
    if let Some(msg) = &app.message {
        lines.push(Line::from(msg.clone()));
    }

    Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Keys "))
}
