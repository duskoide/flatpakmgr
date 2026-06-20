pub mod input;
pub mod mode;
pub mod tabs;

use std::time::Instant;
use tokio::sync::mpsc;

use crate::app::mode::{Focus, Mode, Tab};
use crate::app::tabs::apps::AppsTab;
use crate::flatpak_service::job::JobManager;
use crate::flatpak_service::types::{AppDetail, AppRef, HistoryEntry, Remote, SearchHit};

#[derive(Debug, Clone)]
pub enum RefreshMsg {
    Apps(Vec<AppRef>),
    AppDetail {
        app_ref: AppRef,
        detail: crate::flatpak_service::Result<AppDetail>,
    },
    Runtimes(Vec<AppRef>),
    Remotes(Vec<Remote>),
    History(Vec<HistoryEntry>),
    SearchResults {
        token: u64,
        results: crate::flatpak_service::Result<Vec<SearchHit>>,
    },
}

#[derive(Debug, Clone)]
pub enum Toast {
    Info(String),
    Error(String),
}

pub struct App {
    pub mode: Mode,
    pub tab: Tab,
    pub focus: Focus,
    pub apps: AppsTab,
    pub jobs: JobManager,
    pub toast: Option<(Toast, Instant)>,
    pub should_quit: bool,
    pub last_width: u16,
    pub last_height: u16,
    pub job_rx: mpsc::Receiver<crate::flatpak_service::job::JobEvent>,
    pub refresh_rx: mpsc::Receiver<RefreshMsg>,
    pub refresh_tx: mpsc::Sender<RefreshMsg>,
}

impl App {
    pub fn new() -> Self {
        let (refresh_tx, refresh_rx) = mpsc::channel(32);
        let (job_tx, job_rx) = mpsc::channel(256);
        Self {
            mode: Mode::Normal,
            tab: Tab::Apps,
            focus: Focus::List,
            apps: AppsTab::default(),
            jobs: JobManager::new(job_tx),
            toast: None,
            should_quit: false,
            last_width: 0,
            last_height: 0,
            job_rx,
            refresh_rx,
            refresh_tx,
        }
    }

    pub fn set_toast(&mut self, toast: Toast) {
        self.toast = Some((toast, Instant::now()));
    }

    pub fn clear_expired_toast(&mut self) {
        if let Some((_, t)) = &self.toast {
            if t.elapsed().as_secs() > 5 {
                self.toast = None;
            }
        }
    }
}

use crate::flatpak_service::FlatpakService;
use crate::flatpak_service::job::run_flatpak_job;
use crate::flatpak_service::types::Installation;

pub fn start_apps_refresh(app: &mut App) {
    app.apps.loading = true;
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        let svc = FlatpakService::new();
        let msg = match svc.list_apps(None).await {
            Ok(items) => RefreshMsg::Apps(items),
            Err(_) => RefreshMsg::Apps(Vec::new()),
        };
        let _ = tx.send(msg).await;
    });
}

pub fn start_app_detail_refresh(app: &mut App, app_ref: AppRef) {
    app.apps.detail_loading = true;
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        let svc = FlatpakService::new();
        let detail = svc.info(app_ref.clone()).await;
        let _ = tx.send(RefreshMsg::AppDetail { app_ref, detail }).await;
    });
}

pub fn start_update(app: &mut App, ref_: Option<String>, inst: Installation) {
    let (desc, cmd) = FlatpakService::new().update_cmd(ref_.as_deref(), inst);
    app.jobs.spawn(desc.clone(), move |id, tx| {
        tokio::spawn(run_flatpak_job(id, desc, cmd, tx))
    });
}

pub fn start_uninstall(app: &mut App, ref_: String, inst: Installation, delete_data: bool) {
    let (desc, cmd) = FlatpakService::new().uninstall_cmd(&ref_, inst, delete_data);
    app.jobs.spawn(desc.clone(), move |id, tx| {
        tokio::spawn(run_flatpak_job(id, desc, cmd, tx))
    });
}
