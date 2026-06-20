use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Installation {
    System,
    User,
}

impl std::fmt::Display for Installation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Installation::System => write!(f, "system"),
            Installation::User => write!(f, "user"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Kind {
    App,
    Runtime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppRef {
    pub name: String,
    pub description: String,
    pub id: String,
    pub version: String,
    pub branch: String,
    pub arch: String,
    pub origin: String,
    pub installation: Installation,
    pub size_bytes: u64,
    pub ref_: String,
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Remote {
    pub name: String,
    pub title: String,
    pub url: String,
    pub installation: Installation,
    pub disabled: bool,
    pub priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permission {
    pub table: String,
    pub entries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDetail {
    pub basic: AppRef,
    pub runtime: Option<String>,
    pub sdk: Option<String>,
    pub license: Option<String>,
    pub installed_size: u64,
    pub commit: String,
    pub subject: String,
    pub date: Option<DateTime<Utc>>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryEntry {
    pub time: DateTime<Utc>,
    pub ref_: String,
    pub operation: String,
    pub user: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub name: String,
    pub id: String,
    pub description: String,
    pub version: String,
    pub branch: String,
    pub remotes: Vec<String>,
}
