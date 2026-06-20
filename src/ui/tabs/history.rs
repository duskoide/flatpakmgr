use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use crate::app::App;
use crate::app::tabs::TabState;

pub fn draw_history(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let block = Block::default()
        .title("History")
        .borders(Borders::ALL)
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    let items: Vec<ListItem> = app
        .history
        .items
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let style = if i == app.history.cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<16}", h.time.format("%Y-%m-%d %H:%M")),
                    style,
                ),
                Span::raw("  "),
                Span::styled(format!("{:<10}", h.operation), style),
                Span::raw("  "),
                Span::styled(h.ref_.clone(), style),
            ]))
        })
        .collect();
    let mut state = ListState::default();
    state.select(app.history.selected());
    frame.render_stateful_widget(List::new(items).block(block), area, &mut state);
}
