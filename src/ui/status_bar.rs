use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let left = format!("{} apps", app.apps.items.len());
    let center = app.jobs.handles().iter()
        .find(|j| j.status == crate::flatpak_service::job::JobStatus::Running)
        .map(|j| format!("⚙ {}", j.description))
        .unwrap_or_default();
    let right = "? help  q quit";
    let total_width = area.width as usize;
    let line = format!(
        "{:<24}{:^width$}{:>16}",
        left,
        center,
        right,
        width = total_width.saturating_sub(40)
    );
    frame.render_widget(
        Paragraph::new(Line::from(line)).style(Style::default().bg(Color::DarkGray).fg(Color::White)),
        area,
    );
}
