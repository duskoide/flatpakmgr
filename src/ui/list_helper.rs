use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, layout::Rect,
};

/// Render a virtualized list that only creates ListItems for visible rows.
/// Returns the offset for the caller to track if needed.
pub fn draw_virtual_list<F>(
    frame: &mut Frame,
    area: Rect,
    block: Block,
    total: usize,
    cursor: usize,
    offset: &mut usize,
    make_item: F,
) where
    F: Fn(usize, usize) -> ListItem<'static>,
{
    let inner_height = area.height.saturating_sub(2) as usize; // subtract borders
    if total == 0 {
        frame.render_widget(Paragraph::new("No items").block(block), area);
        return;
    }

    // Adjust offset to keep cursor visible
    if cursor < *offset {
        *offset = cursor;
    } else if cursor >= *offset + inner_height {
        *offset = cursor.saturating_sub(inner_height) + 1;
    }

    let visible = (*offset..(*offset + inner_height).min(total))
        .map(|i| make_item(i, cursor))
        .collect::<Vec<_>>();

    let mut state = ListState::default();
    state.select(Some(cursor.saturating_sub(*offset)));
    frame.render_stateful_widget(List::new(visible).block(block), area, &mut state);
}

pub fn item_style(idx: usize, cursor: usize) -> Style {
    if idx == cursor {
        Style::default().add_modifier(Modifier::REVERSED)
    } else {
        Style::default()
    }
}
