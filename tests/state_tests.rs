use flatpakmgr::flatpak_service::job::{JobEvent, JobId, JobManager};
use tokio::sync::mpsc;

fn make_manager() -> JobManager {
    let (tx, _rx) = mpsc::channel(64);
    JobManager::new(tx)
}

#[test]
fn job_manager_apply_started() {
    let mut mgr = make_manager();
    let id = JobId(1);
    mgr.apply(&JobEvent::Started { id, description: "test job".into() });
}

#[test]
fn job_manager_apply_progress() {
    let mut mgr = make_manager();
    let id = JobId(1);
    mgr.apply(&JobEvent::Progress { id, pct: Some(50), line: "downloading...".into() });
}

#[test]
fn job_manager_apply_finished() {
    let mut mgr = make_manager();
    let id = JobId(1);
    mgr.apply(&JobEvent::Finished { id });
}

#[test]
fn job_manager_apply_failed() {
    let mut mgr = make_manager();
    let id = JobId(1);
    mgr.apply(&JobEvent::Failed { id, msg: "something broke".into() });
}

#[test]
fn job_manager_any_running_empty() {
    let mgr = make_manager();
    assert!(!mgr.any_running());
}

#[test]
fn job_manager_handles_empty() {
    let mgr = make_manager();
    assert!(mgr.handles().is_empty());
}
