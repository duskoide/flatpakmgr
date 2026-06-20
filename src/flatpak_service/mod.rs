pub mod error;
pub mod job;
pub mod parse;
pub mod types;

pub use error::{FlatpakError, Result};
pub use parse::{parse_history, parse_info, parse_list, parse_permissions, parse_remotes};
pub use types::*;

#[derive(Debug, Clone)]
pub struct FlatpakService;

impl FlatpakService {
    pub fn new() -> Self {
        Self
    }
}
