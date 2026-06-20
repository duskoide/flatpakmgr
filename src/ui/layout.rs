use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Tabs},
    Frame,
};
use crate::app::mode::{Mode, Tab};
use crate::app::App;

pub fn layout(frame: &mut Frame, app: &App, draw_content: impl FnOnce(&mut Frame, &App, Rect)) {
    let size = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);
    draw_tab_bar(frame, app, chunks[0]);
    draw_content(frame, app, chunks[1]);
    crate::ui::status_bar::draw(frame, app, chunks[2]);
    crate::ui::toast::draw(frame, app);
    if let Mode::Modal(modal) = &app.mode {
        crate::ui::modals::draw(frame, app, modal);
    }
}

fn draw_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::all().iter().map(|t| Line::from(t.title())).collect();
    let tabs = Tabs::new(titles)
        .select(app.tab as usize)
        .block(Block::default().borders(Borders::ALL).title("flatpakmgr"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, area);
}
