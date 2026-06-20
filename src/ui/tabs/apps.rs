use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};
use crate::app::mode::Focus;
use crate::app::App;

pub fn draw_apps(frame: &mut Frame, app: &App, area: Rect, focus: Focus) {
    if app.last_width < 100 {
        draw_list(frame, app, area, focus == Focus::List || focus == Focus::Tabs);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);
        draw_list(frame, app, chunks[0], focus == Focus::List || focus == Focus::Tabs);
        draw_detail(frame, app, chunks[1], focus == Focus::Detail);
    }
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let block = Block::default()
        .title("Apps")
        .borders(Borders::ALL)
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    let items: Vec<ListItem> = app
        .apps
        .filtered()
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let style = if i == app.apps.cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<24}", a.name), style),
                Span::raw("  "),
                Span::styled(a.version.clone(), style),
            ]))
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.apps.cursor));
    frame.render_stateful_widget(List::new(items).block(block), area, &mut state);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let block = Block::default()
        .title("Detail")
        .borders(Borders::ALL)
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    let text = if app.apps.detail_loading {
        vec![Line::from("Loading…")]
    } else if let Some(d) = &app.apps.detail {
        vec![
            Line::from(vec![
                Span::raw("Name: "),
                Span::styled(d.basic.name.clone(), Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(format!("ID:     {}", d.basic.id)),
            Line::from(format!("Version: {}", d.basic.version)),
            Line::from(format!("Branch:  {}", d.basic.branch)),
            Line::from(format!("Origin:  {} ({})", d.basic.origin, d.basic.installation)),
            Line::from(format!("Runtime: {}", d.runtime.as_deref().unwrap_or("-"))),
            Line::from(format!("License: {}", d.license.as_deref().unwrap_or("-"))),
            Line::from(format!("Commit:  {}", d.commit)),
        ]
    } else {
        vec![Line::from("Select an app")]
    };
    frame.render_widget(
        Paragraph::new(text).block(block).wrap(Wrap { trim: true }),
        area,
    );
}
