use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;

/// Renders the user interface.
pub fn render(app: &App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(frame.area());

    let items: Vec<ListItem> = app
        .todos
        .iter()
        .enumerate()
        .map(|(i, todo)| {
            let status = if todo.is_done { "[✓] " } else { "[ ] " };
            let style = if i == app.selected_index {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(format!("{}{}", status, todo.text), style))
        })
        .collect();

    let todo_list =
        List::new(items).block(Block::default().borders(Borders::ALL).title("Todos"));
    frame.render_widget(todo_list, chunks[0]);

    let help_text = " ↓/↑ o j/k: Navegar | Espacio: Completar/Desmarcar | q: Salir ";
    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title(" Atajos "));
    frame.render_widget(help_paragraph, chunks[1]);
}
