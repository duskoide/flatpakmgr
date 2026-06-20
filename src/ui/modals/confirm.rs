use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::mode::ConfirmAction;
use crate::app::App;
use super::centered_rect;

pub fn draw(frame: &mut Frame, _app: &App, action: &ConfirmAction) {
    let area = centered_rect(50, 30, frame.area());
    frame.render_widget(Clear, area);
    let msg = match action {
        ConfirmAction::Uninstall { ref_, .. } => format!("Uninstall {}?", ref_),
    };
    let lines = vec![
        Line::from(Span::styled(msg, Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from("y to confirm, Esc to cancel"),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Confirm")),
        area,
    );
}
