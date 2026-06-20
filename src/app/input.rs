use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use crate::app::mode::{ConfirmAction, Focus, Modal, Mode, Tab};
use crate::app::tabs::TabState;
use crate::app::{start_permissions_refresh, start_uninstall, start_update, App};
use crate::flatpak_service::types::Installation;

pub fn handle_input(app: &mut App, event: Event) {
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if matches!(app.mode, Mode::Modal(_)) {
            handle_modal_input(app, key);
            return;
        }
        // Global keys (work from any focus except Search)
        if app.focus != Focus::Search && handle_global_keys(app, key) {
            return;
        }
        match app.focus {
            Focus::List => handle_list_input(app, key),
            Focus::Detail => handle_detail_input(app, key),
            Focus::Search => handle_search_input(app, key),
        }
    }
}

/// Returns true if the key was handled
fn handle_global_keys(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') => { app.should_quit = true; true }
        KeyCode::Char('?') => { app.mode = Mode::Modal(Modal::Help); true }
        KeyCode::Char('J') => { app.mode = Mode::Modal(Modal::Jobs); true }
        KeyCode::Char('1') => { switch_tab(app, Tab::Apps); true }
        KeyCode::Char('2') => { switch_tab(app, Tab::Runtimes); true }
        KeyCode::Char('3') => { switch_tab(app, Tab::Remotes); true }
        KeyCode::Char('4') => { switch_tab(app, Tab::History); true }
        KeyCode::Char('5') => { switch_tab(app, Tab::Install); true }
        _ => false,
    }
}

fn switch_tab(app: &mut App, tab: Tab) {
    app.tab = tab;
    app.focus = Focus::List;
    match tab {
        Tab::Apps => crate::app::start_apps_refresh(app),
        Tab::Runtimes => crate::app::start_runtimes_refresh(app),
        Tab::Remotes => crate::app::start_remotes_refresh(app),
        Tab::History => crate::app::start_history_refresh(app),
        Tab::Install => {},
    }
}

fn handle_list_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('r') => match app.tab {
            Tab::Apps => crate::app::start_apps_refresh(app),
            Tab::Runtimes => crate::app::start_runtimes_refresh(app),
            Tab::Remotes => crate::app::start_remotes_refresh(app),
            Tab::History => crate::app::start_history_refresh(app),
            _ => {}
        },
        KeyCode::Char('j') | KeyCode::Down => match app.tab {
            Tab::Apps => {
                app.apps.move_cursor(1);
                if let Some(a) = app.apps.selected_ref() {
                    crate::app::start_app_detail_refresh(app, a.clone());
                }
            }
            Tab::Runtimes => app.runtimes.move_cursor(1),
            Tab::Remotes => app.remotes.move_cursor(1),
            Tab::History => app.history.move_cursor(1),
            Tab::Install => app.install.move_cursor(1),
        },
        KeyCode::Char('k') | KeyCode::Up => match app.tab {
            Tab::Apps => {
                app.apps.move_cursor(-1);
                if let Some(a) = app.apps.selected_ref() {
                    crate::app::start_app_detail_refresh(app, a.clone());
                }
            }
            Tab::Runtimes => app.runtimes.move_cursor(-1),
            Tab::Remotes => app.remotes.move_cursor(-1),
            Tab::History => app.history.move_cursor(-1),
            Tab::Install => app.install.move_cursor(-1),
        },
        KeyCode::Char('u') => {
            if let Some(a) = app.apps.selected_ref() {
                let ref_ = a.ref_.clone();
                let inst = a.installation.clone();
                start_update(app, Some(ref_), inst);
            }
        }
        KeyCode::Char('U') => {
            let inst = app.apps.selected_ref().map(|a| a.installation.clone()).unwrap_or(Installation::System);
            start_update(app, None, inst);
        }
        KeyCode::Char('d') => {
            if let Some(a) = app.apps.selected_ref() {
                app.mode = Mode::Modal(Modal::Confirm(ConfirmAction::Uninstall {
                    ref_: a.ref_.clone(),
                    inst: a.installation.clone(),
                }));
            }
        }
        KeyCode::Char('e') => {
            if app.tab == Tab::Remotes
                && let Some(r) = app.remotes.selected_remote() {
                    let name = r.name.clone();
                    let inst = r.installation.clone();
                    let enable = r.disabled;
                    crate::app::start_remote_toggle(app, name, inst, enable);
                }
        }
        KeyCode::Char('p') => {
            if let Some(a) = app.apps.selected_ref() {
                let id = a.id.clone();
                app.mode = Mode::Modal(Modal::Permissions { id: id.clone() });
                start_permissions_refresh(app, id);
            }
        }
        KeyCode::Char('/') => {
            if app.tab == Tab::Install {
                app.focus = Focus::Search;
            }
        }
        KeyCode::Tab => {
            if app.tab == Tab::Apps {
                app.focus = Focus::Detail;
            } else if app.tab == Tab::Install {
                app.focus = Focus::Search;
            }
        }
        KeyCode::BackTab | KeyCode::Esc => {}
        _ => {}
    }
}

fn handle_detail_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Esc => app.focus = Focus::List,
        _ => {}
    }
}

fn handle_search_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app.install.query.push(c);
            crate::app::start_search(app);
        }
        KeyCode::Backspace => {
            app.install.query.pop();
            crate::app::start_search(app);
        }
        KeyCode::Esc => app.focus = Focus::List,
        KeyCode::Tab | KeyCode::BackTab => app.focus = Focus::List,
        KeyCode::Enter => {
            if let Some(hit) = app.install.results.get(app.install.cursor) {
                let remote = hit.remotes.first().cloned().unwrap_or_default();
                let kind = if hit.id.contains(".Runtime") {
                    "runtime"
                } else {
                    "app"
                };
                let ref_ = format!("{}/{}/{}/{}", kind, hit.id, "x86_64", hit.branch);
                let (desc, cmd) = crate::flatpak_service::FlatpakService::new()
                    .install_cmd(&remote, &ref_, Installation::User);
                app.jobs.spawn(desc.clone(), move |id, tx| {
                    tokio::spawn(crate::flatpak_service::job::run_flatpak_job(
                        id, desc, cmd, tx,
                    ))
                });
            }
        }
        KeyCode::Down => app.install.move_cursor(1),
        KeyCode::Up => app.install.move_cursor(-1),
        _ => {}
    }
}

fn handle_modal_input(app: &mut App, key: KeyEvent) {
    match &app.mode {
        Mode::Modal(Modal::Confirm(ConfirmAction::Uninstall { ref_, inst })) => {
            if key.code == KeyCode::Enter {
                let ref_ = ref_.clone();
                let inst = inst.clone();
                start_uninstall(app, ref_, inst, false);
            }
            app.mode = Mode::Normal;
        }
        _ => {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                app.mode = Mode::Normal;
            }
        }
    }
}
