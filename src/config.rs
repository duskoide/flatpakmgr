#[derive(Debug, Clone, Default)]
pub struct Config;

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self)
    }
}
