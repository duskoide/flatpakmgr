use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
    Frame,
};
use crate::app::App;
use crate::flatpak_service::job::JobStatus;
use super::centered_rect;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = centered_rect(80, 80, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).title("Jobs");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.jobs.handles().is_empty() {
        frame.render_widget(Paragraph::new("No jobs."), inner);
        return;
    }

    let rows: Vec<Rect> = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3); app.jobs.handles().len()])
        .split(inner)
        .to_vec();

    for (i, job) in app.jobs.handles().iter().enumerate() {
        let pct = job.log.iter().rev().find_map(|l| crate::flatpak_service::parse::parse_progress_line(l));
        let label = match &job.status {
            JobStatus::Running => format!("{} ({})", job.description, pct.map(|p| format!("{}%", p)).unwrap_or_else(|| "working".into())),
            JobStatus::Finished => format!("{} - done", job.description),
            JobStatus::Failed => format!("{} - failed", job.description),
        };
        let color = match &job.status {
            JobStatus::Running => Color::Blue,
            JobStatus::Finished => Color::Green,
            JobStatus::Failed => Color::Red,
        };
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(job.description.clone()))
            .gauge_style(Style::default().fg(color))
            .percent(pct.unwrap_or(0))
            .label(label);
        frame.render_widget(gauge, rows[i]);
    }
}
