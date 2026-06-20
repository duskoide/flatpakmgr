use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::app::App;
use super::centered_rect;

pub fn draw(frame: &mut Frame, app: &App, id: &str) {
    let area = centered_rect(70, 60, frame.area());
    frame.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(area);

    let header = Paragraph::new(vec![
        Line::from(Span::styled(
            format!(" Permissions: {} ", id),
            Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD),
        )),
        Line::from(""),
    ]);
    frame.render_widget(header, chunks[0]);

    let mut lines: Vec<Line> = Vec::new();
    if app.apps.permissions.is_empty() {
        lines.push(Line::from("No permissions loaded"));
    } else {
        for perm in &app.apps.permissions {
            lines.push(Line::from(Span::styled(
                format!("[{}]", perm.table),
                Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD),
            )));
            for entry in &perm.entries {
                lines.push(Line::from(format!("  {}", entry)));
            }
            lines.push(Line::from(""));
        }
    }

    frame.render_widget(
        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title("Permissions"))
            .wrap(Wrap { trim: true }),
        chunks[1],
    );
}
