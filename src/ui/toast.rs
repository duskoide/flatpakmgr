use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::app::{App, Toast};

pub fn draw(frame: &mut Frame, app: &App) {
    if let Some((toast, _)) = &app.toast {
        let (msg, color) = match toast {
            Toast::Info(s) => (s.as_str(), Color::Blue),
            Toast::Error(s) => (s.as_str(), Color::Red),
        };
        let area = toast_area(frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(
            Paragraph::new(Line::from(msg))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(color)),
                )
                .alignment(Alignment::Center),
            area,
        );
    }
}

fn toast_area(root: Rect) -> Rect {
    let width = (root.width as f32 * 0.6).min(60.0) as u16;
    Rect {
        x: root.width.saturating_sub(width + 2),
        y: 1,
        width,
        height: 3,
    }
}
