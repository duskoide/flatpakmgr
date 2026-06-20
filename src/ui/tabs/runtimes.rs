use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use crate::app::App;
use crate::app::tabs::TabState;

pub fn draw_runtimes(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let block = Block::default()
        .title("Runtimes")
        .borders(Borders::ALL)
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    let items: Vec<ListItem> = app
        .runtimes
        .items
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let style = if i == app.runtimes.cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<40}", r.name), style),
                Span::styled(r.version.clone(), style),
            ]))
        })
        .collect();
    let mut state = ListState::default();
    state.select(app.runtimes.selected());
    frame.render_stateful_widget(List::new(items).block(block), area, &mut state);
}
