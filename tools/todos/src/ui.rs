use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::{App, Focus, InputMode};

const ACTIVE_BORDER: Color = Color::Cyan;
const INACTIVE_BORDER: Color = Color::DarkGray;

/// Renders the user interface: un layout de paneles al estilo lazygit/lazydocker
/// (lista de secciones a la izquierda, tareas de la sección activa a la derecha,
/// y una barra de estado/ayuda abajo).
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
            let text = format!("{} ({done}/{})", section.name, section.todos.len());
            ListItem::new(Span::styled(text, row_style(selected, focused)))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(focused))
        .title(" Secciones ");
    frame.render_widget(List::new(items).block(block), area);
}

fn render_todos(app: &App, frame: &mut Frame, area: Rect) {
    let focused = app.focus == Focus::Todos;
    let section = app.selected_section();

    let title = match section {
        Some(section) => format!(" Todos — {} ", section.name),
        None => " Todos ".to_string(),
    };

    let items: Vec<ListItem> = section
        .map(|section| {
            section
                .todos
                .iter()
                .enumerate()
                .map(|(i, todo)| {
                    let selected = i == app.selected_todo;
                    let status = if todo.is_done { "[✓] " } else { "[ ] " };
                    let text = format!("{status}{}", todo.text);
                    ListItem::new(Span::styled(text, row_style(selected, focused)))
                })
                .collect()
        })
        .unwrap_or_default();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style(focused))
        .title(title);
    frame.render_widget(List::new(items).block(block), area);
}

fn border_style(focused: bool) -> Style {
    if focused {
        Style::default().fg(ACTIVE_BORDER)
    } else {
        Style::default().fg(INACTIVE_BORDER)
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
            frame.render_widget(input_paragraph("Nueva sección", &app.input_buffer), area);
        }
        InputMode::AddTodo => {
            frame.render_widget(input_paragraph("Nueva tarea", &app.input_buffer), area);
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
            .title(" Enter: confirmar · Esc: cancelar "),
    )
}

fn help_paragraph(app: &App) -> Paragraph<'static> {
    let key_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let keys = Line::from(vec![
        Span::styled("Tab/h/l", key_style),
        Span::raw(" panel  "),
        Span::styled("j/k", key_style),
        Span::raw(" mover  "),
        Span::styled("a", key_style),
        Span::raw(" agregar  "),
        Span::styled("espacio", key_style),
        Span::raw(" completar  "),
        Span::styled("s", key_style),
        Span::raw(" guardar  "),
        Span::styled("q", key_style),
        Span::raw(" salir"),
    ]);

    let mut lines = vec![keys];
    if let Some(msg) = &app.message {
        lines.push(Line::from(msg.clone()));
    }

    Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Atajos "))
}
