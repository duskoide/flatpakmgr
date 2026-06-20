use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::flatpak_service::{parse::parse_progress_line, FlatpakError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(pub u64);

#[derive(Debug, Clone)]
pub enum JobEvent {
    Started { id: JobId, description: String },
    Progress { id: JobId, pct: Option<u16>, line: String },
    Finished { id: JobId },
    Failed { id: JobId, msg: String },
}

#[derive(Debug, Clone)]
pub struct JobHandle {
    pub id: JobId,
    pub description: String,
    pub status: JobStatus,
    pub log: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Finished,
    Failed,
}

pub struct JobManager {
    next_id: u64,
    tx: mpsc::Sender<JobEvent>,
    jobs: Vec<JobHandle>,
    running: tokio::task::JoinSet<(JobId, Result<()>)>,
}

impl JobManager {
    pub fn new(tx: mpsc::Sender<JobEvent>) -> Self {
        Self {
            next_id: 1,
            tx,
            jobs: Vec::new(),
            running: tokio::task::JoinSet::new(),
        }
    }

    pub fn spawn<F>(&mut self, description: String, work: F) -> JobId
    where
        F: FnOnce(JobId, mpsc::Sender<JobEvent>) -> tokio::task::JoinHandle<Result<()>>,
    {
        let id = JobId(self.next_id);
        self.next_id += 1;
        self.jobs.push(JobHandle {
            id,
            description: description.clone(),
            status: JobStatus::Running,
            log: Vec::new(),
        });
        let _ = self.tx.try_send(JobEvent::Started { id, description });
        let handle = work(id, self.tx.clone());
        self.running.spawn(async move {
            let result = handle.await.unwrap_or_else(|e| Err(FlatpakError::Io(e.into())));
            (id, result)
        });
        id
    }

    pub fn apply(&mut self, event: &JobEvent) {
        match event {
            JobEvent::Progress { id, line, .. } => {
                if let Some(j) = self.jobs.iter_mut().find(|j| j.id == *id) {
                    j.log.push(line.clone());
                }
            }
            JobEvent::Finished { id } => {
                if let Some(j) = self.jobs.iter_mut().find(|j| j.id == *id) {
                    j.status = JobStatus::Finished;
                }
            }
            JobEvent::Failed { id, .. } => {
                if let Some(j) = self.jobs.iter_mut().find(|j| j.id == *id) {
                    j.status = JobStatus::Failed;
                }
            }
            _ => {}
        }
    }

    pub fn handles(&self) -> &[JobHandle] {
        &self.jobs
    }

    pub fn any_running(&self) -> bool {
        self.jobs.iter().any(|j| j.status == JobStatus::Running)
    }
}

pub async fn run_flatpak_job(
    id: JobId,
    description: String,
    mut cmd: Command,
    tx: mpsc::Sender<JobEvent>,
) -> Result<()> {
    let _ = tx.send(JobEvent::Started { id, description }).await;

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    let tx_stdout = tx.clone();
    let stdout_handle = tokio::spawn(read_stream(id, stdout, tx_stdout));
    let tx_stderr = tx.clone();
    let stderr_handle = tokio::spawn(read_stream(id, stderr, tx_stderr));

    let (status, _, _) = tokio::join!(child.wait(), stdout_handle, stderr_handle);

    let code = status?.code().unwrap_or(-1);
    if code != 0 {
        let _ = tx.send(JobEvent::Failed { id, msg: format!("exit code {}", code) }).await;
        return Err(FlatpakError::Cli { code, stderr: String::new() });
    }
    let _ = tx.send(JobEvent::Finished { id }).await;
    Ok(())
}

async fn read_stream<R: tokio::io::AsyncRead + Unpin>(
    id: JobId,
    reader: R,
    tx: mpsc::Sender<JobEvent>,
) {
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let pct = parse_progress_line(&line);
        let _ = tx.send(JobEvent::Progress { id, pct, line }).await;
    }
}
