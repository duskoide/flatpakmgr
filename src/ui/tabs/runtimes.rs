use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Block,
    Frame,
};
use crate::app::App;
use crate::ui::list_helper::{draw_virtual_list, item_style};

pub fn draw_runtimes(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let block = Block::default()
        .title(format!("Runtimes ({})", app.runtimes.items.len()))
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(if focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let total = app.runtimes.items.len();
    let cursor = app.runtimes.cursor;
    let mut offset = 0;

    draw_virtual_list(frame, area, block, total, cursor, &mut offset, |i, cursor| {
        let r = &app.runtimes.items[i];
        let style = item_style(i, cursor);
        ratatui::widgets::ListItem::new(Line::from(vec![
            Span::styled(format!("{:<40}", r.name), style),
            Span::styled(r.version.clone(), style),
        ]))
    });
}
