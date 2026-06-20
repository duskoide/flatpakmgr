pub mod confirm;
pub mod help;
pub mod jobs;
pub mod permissions;

use ratatui::Frame;
use crate::app::mode::Modal;
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, modal: &Modal) {
    match modal {
        Modal::Help => help::draw(frame, app),
        Modal::Jobs => jobs::draw(frame, app),
        Modal::Confirm(action) => confirm::draw(frame, app, action),
        Modal::Permissions { id } => permissions::draw(frame, app, id),
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
