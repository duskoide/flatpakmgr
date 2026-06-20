use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::App;
use super::centered_rect;

pub fn draw(frame: &mut Frame, _app: &App, id: &str) {
    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);
    let lines = vec![
        Line::from(Span::styled(
            format!(" Permissions: {} ", id),
            Style::default().fg(Color::Yellow),
        )),
        Line::from(""),
        Line::from("Loading permissions…"),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Permissions")),
        area,
    );
}
