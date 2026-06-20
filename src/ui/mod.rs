pub mod layout;
pub mod modals;
pub mod status_bar;
pub mod tabs;
pub mod toast;

use ratatui::Frame;
use crate::app::mode::Tab;
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    layout::layout(frame, app, |frame, app, area| match app.tab {
        Tab::Apps => tabs::apps::draw_apps(frame, app, area, app.focus),
        _ => {
            let text = ratatui::text::Text::from(format!("{} tab not yet implemented", app.tab.title()));
            frame.render_widget(ratatui::widgets::Paragraph::new(text), area);
        }
    });
}
