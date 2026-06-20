pub mod error;
pub mod job;
pub mod parse;
pub mod types;

pub use error::{FlatpakError, Result};
pub use parse::{parse_history, parse_info, parse_list, parse_permissions, parse_remotes};
pub use types::*;

use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct FlatpakService;

impl FlatpakService {
    pub fn new() -> Self {
        Self
    }

    async fn run_cmd(&self, cmd: &str) -> Result<String> {
        tracing::debug!("running: {}", cmd);
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .await?;
        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(FlatpakError::Cli { code, stderr });
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn list_installed(
        &self,
        inst: Option<&Installation>,
        kind: Kind,
    ) -> Result<Vec<AppRef>> {
        let inst_flag = match inst {
            Some(Installation::User) => "--user",
            _ => "--system",
        };
        let kind_flag = match kind {
            Kind::App => "--app",
            Kind::Runtime => "--runtime",
        };
        let cmd = format!("flatpak list {} {} --columns=name,description,application,version,branch,arch,origin,installation,size,ref", inst_flag, kind_flag);
        let out = self.run_cmd(&cmd).await?;
        parse_list(&out, kind)
    }

    pub async fn list_apps(&self, inst: Option<&Installation>) -> Result<Vec<AppRef>> {
        self.list_installed(inst, Kind::App).await
    }

    pub async fn list_runtimes(&self, inst: Option<&Installation>) -> Result<Vec<AppRef>> {
        self.list_installed(inst, Kind::Runtime).await
    }

    pub async fn info(&self, basic: AppRef) -> Result<AppDetail> {
        let inst_flag = match basic.installation {
            Installation::User => "--user",
            _ => "--system",
        };
        let cmd = format!(
            "flatpak info {} {}",
            inst_flag, basic.id
        );
        let out = self.run_cmd(&cmd).await?;
        parse_info(&out, basic)
    }

    pub async fn list_remotes(&self, inst: Option<&Installation>) -> Result<Vec<Remote>> {
        let inst_flag = match inst {
            Some(Installation::User) => "--user",
            _ => "--system",
        };
        let cmd = format!(
            "flatpak remotes {} --columns=name,title,url,installation,disabled,priority",
            inst_flag
        );
        let out = self.run_cmd(&cmd).await?;
        parse_remotes(&out)
    }

    pub async fn list_history(&self) -> Result<Vec<HistoryEntry>> {
        let cmd = "flatpak history --columns=time,ref,operation,user".to_string();
        let out = self.run_cmd(&cmd).await?;
        parse_history(&out)
    }

    pub async fn permissions(&self, id: &str) -> Result<Vec<Permission>> {
        let cmd = format!("flatpak info --show-permissions {}", id);
        let out = self.run_cmd(&cmd).await?;
        Ok(parse_permissions(&out))
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchHit>> {
        let cmd = format!(
            "flatpak search {} --columns=name,application,description,version,branch,remotes",
            query
        );
        let out = self.run_cmd(&cmd).await?;
        let mut hits = Vec::new();
        for line in out.lines() {
            if line.is_empty() {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() >= 5 {
                let remotes = if cols.len() > 5 {
                    cols[5].split(',').map(|s| s.to_string()).collect()
                } else {
                    Vec::new()
                };
                hits.push(SearchHit {
                    name: cols[0].to_string(),
                    id: cols[1].to_string(),
                    description: cols[2].to_string(),
                    version: cols[3].to_string(),
                    branch: cols[4].to_string(),
                    remotes,
                });
            }
        }
        Ok(hits)
    }
}
