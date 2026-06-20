pub mod input;
pub mod mode;
pub mod tabs;

use std::time::Instant;
use tokio::sync::mpsc;

use crate::app::mode::{Focus, Mode, Tab};
use crate::app::tabs::apps::AppsTab;
use crate::app::tabs::history::HistoryTab;
use crate::app::tabs::install::InstallTab;
use crate::app::tabs::remotes::RemotesTab;
use crate::app::tabs::runtimes::RuntimesTab;
use crate::flatpak_service::job::JobManager;
use crate::flatpak_service::types::{AppDetail, AppRef, HistoryEntry, Permission, Remote, SearchHit};

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
    Permissions {
        id: String,
        perms: Vec<Permission>,
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
    pub runtimes: RuntimesTab,
    pub remotes: RemotesTab,
    pub history: HistoryTab,
    pub install: InstallTab,
    pub jobs: JobManager,
    pub toast: Option<(Toast, Instant)>,
    pub should_quit: bool,
    pub last_width: u16,
    pub last_height: u16,
    pub job_rx: mpsc::Receiver<crate::flatpak_service::job::JobEvent>,
    pub refresh_rx: mpsc::Receiver<RefreshMsg>,
    pub refresh_tx: mpsc::Sender<RefreshMsg>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
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
            runtimes: RuntimesTab::default(),
            remotes: RemotesTab::default(),
            history: HistoryTab::default(),
            install: InstallTab::default(),
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
        if let Some((_, t)) = &self.toast
            && t.elapsed().as_secs() > 5 {
                self.toast = None;
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

pub fn start_runtimes_refresh(app: &mut App) {
    app.runtimes.loading = true;
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        let svc = FlatpakService::new();
        let msg = match svc.list_runtimes(None).await {
            Ok(items) => RefreshMsg::Runtimes(items),
            Err(_) => RefreshMsg::Runtimes(Vec::new()),
        };
        let _ = tx.send(msg).await;
    });
}

pub fn start_remotes_refresh(app: &mut App) {
    app.remotes.loading = true;
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        let svc = FlatpakService::new();
        let msg = match svc.list_remotes(None).await {
            Ok(items) => RefreshMsg::Remotes(items),
            Err(_) => RefreshMsg::Remotes(Vec::new()),
        };
        let _ = tx.send(msg).await;
    });
}

pub fn start_history_refresh(app: &mut App) {
    app.history.loading = true;
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        let svc = FlatpakService::new();
        let msg = match svc.list_history().await {
            Ok(items) => RefreshMsg::History(items),
            Err(_) => RefreshMsg::History(Vec::new()),
        };
        let _ = tx.send(msg).await;
    });
}

pub fn start_search(app: &mut App) {
    app.install.loading = true;
    app.install.debounce_token += 1;
    let token = app.install.debounce_token;
    let query = app.install.query.clone();
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let svc = FlatpakService::new();
        let results = svc.search(&query).await;
        let _ = tx.send(RefreshMsg::SearchResults { token, results }).await;
    });
}

pub fn start_remote_toggle(app: &mut App, name: String, inst: Installation, enable: bool) {
    let (desc, cmd) = FlatpakService::new().remote_modify_cmd(&name, inst, enable);
    app.jobs.spawn(desc.clone(), move |id, tx| {
        tokio::spawn(run_flatpak_job(id, desc, cmd, tx))
    });
}

pub fn start_permissions_refresh(app: &mut App, id: String) {
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        let svc = FlatpakService::new();
        let perms = svc.permissions(&id).await.unwrap_or_default();
        let _ = tx.send(RefreshMsg::Permissions { id, perms }).await;
    });
}
