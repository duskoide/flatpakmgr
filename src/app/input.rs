use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use crate::app::mode::{ConfirmAction, Focus, Modal, Mode, Tab};
use crate::app::tabs::TabState;
use crate::app::{start_uninstall, start_update, App};
use crate::flatpak_service::types::Installation;

pub fn handle_input(app: &mut App, event: Event) {
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if matches!(app.mode, crate::app::mode::Mode::Modal(_)) {
            handle_modal_input(app, key);
            return;
        }
        match app.focus {
            Focus::Tabs => handle_tab_bar_input(app, key),
            Focus::List => handle_list_input(app, key),
            Focus::Detail => handle_detail_input(app, key),
            Focus::Search => handle_search_input(app, key),
        }
    }
}

fn handle_tab_bar_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.mode = crate::app::mode::Mode::Modal(Modal::Help),
        KeyCode::Char('J') => app.mode = crate::app::mode::Mode::Modal(Modal::Jobs),
        KeyCode::Char('1') => app.tab = Tab::Apps,
        KeyCode::Char('2') => app.tab = Tab::Runtimes,
        KeyCode::Char('3') => app.tab = Tab::Remotes,
        KeyCode::Char('4') => app.tab = Tab::History,
        KeyCode::Char('5') => app.tab = Tab::Install,
        KeyCode::Tab => app.focus = Focus::List,
        _ => {}
    }
}

fn handle_list_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.mode = Mode::Modal(Modal::Help),
        KeyCode::Char('J') => app.mode = Mode::Modal(Modal::Jobs),
        KeyCode::Char('r') => crate::app::start_apps_refresh(app),
        KeyCode::Char('j') | KeyCode::Down => {
            app.apps.move_cursor(1);
            if let Some(a) = app.apps.selected_ref() {
                crate::app::start_app_detail_refresh(app, a.clone());
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.apps.move_cursor(-1);
            if let Some(a) = app.apps.selected_ref() {
                crate::app::start_app_detail_refresh(app, a.clone());
            }
        }
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
        KeyCode::Tab => app.focus = Focus::Detail,
        KeyCode::BackTab => app.focus = Focus::Tabs,
        KeyCode::Esc => app.focus = Focus::Tabs,
        _ => {}
    }
}

fn handle_detail_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Esc | KeyCode::BackTab => app.focus = Focus::List,
        _ => {}
    }
}

fn handle_search_input(_app: &mut App, _key: KeyEvent) {
    // Implemented in Task 21
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
