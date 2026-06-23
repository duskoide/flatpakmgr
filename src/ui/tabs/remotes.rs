use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Block,
    Frame,
};
use crate::app::App;
use crate::ui::list_helper::{draw_virtual_list, item_style};

pub fn draw_remotes(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let block = Block::default()
        .title(format!("Remotes ({})", app.remotes.items.len()))
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let total = app.remotes.items.len();
    let cursor = app.remotes.cursor;
    let mut offset = 0;

    draw_virtual_list(frame, area, block, total, cursor, &mut offset, |i, cursor| {
        let r = &app.remotes.items[i];
        let style = item_style(i, cursor);
        let status = if r.disabled { "[disabled]" } else { "[enabled]" };
        ratatui::widgets::ListItem::new(Line::from(vec![
            Span::styled(format!("{:<24}", r.name), style),
            Span::raw("  "),
            Span::styled(status.to_string(), style),
            Span::raw("  "),
            Span::styled(r.url.clone(), style),
        ]))
    });
}
