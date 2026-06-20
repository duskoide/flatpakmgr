use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use crate::app::App;
use crate::app::tabs::TabState;

pub fn draw_remotes(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let block = Block::default()
        .title("Remotes")
        .borders(Borders::ALL)
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    let items: Vec<ListItem> = app
        .remotes
        .items
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let style = if i == app.remotes.cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            let status = if r.disabled { "[disabled]" } else { "[enabled]" };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<24}", r.name), style),
                Span::raw("  "),
                Span::styled(status.to_string(), style),
                Span::raw("  "),
                Span::styled(r.url.clone(), style),
            ]))
        })
        .collect();
    let mut state = ListState::default();
    state.select(app.remotes.selected());
    frame.render_stateful_widget(List::new(items).block(block), area, &mut state);
}
