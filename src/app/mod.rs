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
