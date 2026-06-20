use thiserror::Error;

#[derive(Error, Debug)]
pub enum FlatpakError {
    #[error("flatpak exited with code {code}: {stderr}")]
    Cli { code: i32, stderr: String },
    #[error("failed to parse line: {msg}\n  line: {line}")]
    Parse { line: String, msg: String },
    #[error("not found: {0}")]
    NotFound(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, FlatpakError>;
