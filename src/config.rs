use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub default_installation: Option<String>,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = config_path()?;
        if path.exists() {
            let text = fs::read_to_string(&path)?;
            Ok(serde_json::from_str(&text)?)
        } else {
            Ok(Self::default())
        }
    }

    #[allow(dead_code)]
    pub fn save(&self) -> anyhow::Result<()> {
        let path = config_path()?;
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

fn config_path() -> anyhow::Result<PathBuf> {
    let dirs = ProjectDirs::from("", "", "flatpakmgr")
        .ok_or_else(|| anyhow::anyhow!("project dirs"))?;
    Ok(dirs.config_dir().join("config.json"))
}
