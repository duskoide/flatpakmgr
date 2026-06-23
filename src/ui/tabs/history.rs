use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Block,
    Frame,
};
use crate::app::App;
use crate::ui::list_helper::{draw_virtual_list, item_style};

pub fn draw_history(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let block = Block::default()
        .title(format!("History ({})", app.history.items.len()))
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let total = app.history.items.len();
    let cursor = app.history.cursor;
    let mut offset = 0;

    draw_virtual_list(frame, area, block, total, cursor, &mut offset, |i, cursor| {
        let h = &app.history.items[i];
        let style = item_style(i, cursor);
        ratatui::widgets::ListItem::new(Line::from(vec![
            Span::styled(
                format!("{:<16}", h.time.format("%Y-%m-%d %H:%M")),
                style,
            ),
            Span::raw("  "),
            Span::styled(format!("{:<10}", h.operation), style),
            Span::raw("  "),
            Span::styled(h.ref_.clone(), style),
        ]))
    });
}
