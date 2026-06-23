use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use crate::app::App;
use crate::app::tabs::TabState;

pub fn draw_install(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    let search_focused = focused && crate::app::mode::Focus::Search == app.focus;
    let list_focused = focused && crate::app::mode::Focus::Search != app.focus;

    let search_block = Block::default()
        .title("Search")
        .borders(Borders::ALL)
        .border_style(if search_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });
    let search_text = format!("{}_", app.install.query);
    frame.render_widget(
        Paragraph::new(search_text).block(search_block),
        chunks[0],
    );

    let list_block = Block::default()
        .title(format!("Results ({})", app.install.results.len()))
        .borders(Borders::ALL)
        .border_style(if list_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        });

    let inner_height = chunks[1].height.saturating_sub(2) as usize; // subtract borders
    let total = app.install.results.len();
    if total == 0 {
        frame.render_widget(
            Paragraph::new("No results").block(list_block),
            chunks[1],
        );
        return;
    }

    let cursor = app.install.cursor;
    let offset = app.install.offset;
    let offset = if cursor < offset {
        cursor
    } else if cursor >= offset + inner_height {
        cursor.saturating_sub(inner_height) + 1
    } else {
        offset
    };

    let visible = app.install.results.iter()
        .skip(offset)
        .take(inner_height)
        .enumerate()
        .map(|(i, r)| {
            let idx = offset + i;
            let style = if idx == cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<32}", r.name), style),
                Span::raw("  "),
                Span::styled(r.id.clone(), style),
            ]))
        })
        .collect::<Vec<_>>();

    let mut state = ListState::default();
    state.select(Some(cursor.saturating_sub(offset)));
    frame.render_stateful_widget(
        List::new(visible).block(list_block),
        chunks[1],
        &mut state,
    );
}
