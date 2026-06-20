pub mod layout;
pub mod modals;
pub mod status_bar;
pub mod tabs;
pub mod toast;

use ratatui::Frame;
use crate::app::mode::{Focus, Tab};
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    layout::layout(frame, app, |frame, app, area| match app.tab {
        Tab::Apps => tabs::apps::draw_apps(frame, app, area, app.focus),
        Tab::Runtimes => tabs::runtimes::draw_runtimes(frame, app, area, app.focus == Focus::List),
        Tab::Remotes => tabs::remotes::draw_remotes(frame, app, area, app.focus == Focus::List),
        Tab::History => tabs::history::draw_history(frame, app, area, app.focus == Focus::List),
        Tab::Install => tabs::install::draw_install(frame, app, area, app.focus == Focus::List || app.focus == Focus::Search),
    });
}
