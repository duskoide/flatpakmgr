use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::App;
use super::centered_rect;

pub fn draw(frame: &mut Frame, _app: &App) {
    let area = centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);
    let lines = vec![
        Line::from(Span::styled(" Help ", Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from("q       Quit"),
        Line::from("?       Toggle help"),
        Line::from("J       Job monitor"),
        Line::from("1-5     Switch tabs"),
        Line::from("j/k     Navigate list"),
        Line::from("r       Refresh"),
        Line::from("u/U     Update app/all"),
        Line::from("d       Uninstall app"),
        Line::from("p       Show permissions"),
        Line::from("e       Toggle remote"),
        Line::from("/       Search (Install tab)"),
        Line::from("Tab     Next pane"),
        Line::from("Esc     Back / close"),
    ];
    frame.render_widget(
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Help")),
        area,
    );
}
