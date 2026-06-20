use directories::ProjectDirs;
use std::fs;
use std::sync::Mutex;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

pub fn init(verbose: bool) -> anyhow::Result<()> {
    let dirs = ProjectDirs::from("", "", "flatpakmgr")
        .ok_or_else(|| anyhow::anyhow!("could not determine project dirs"))?;
    let log_dir = dirs.cache_dir();
    fs::create_dir_all(log_dir)?;
    let log_file = log_dir.join("flatpakmgr.log");

    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    let fmt_layer = fmt::layer()
        .with_writer(Mutex::new(file))
        .with_ansi(false);
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug"));

    let subscriber = Registry::default().with(env_filter).with(fmt_layer);

    if verbose {
        let stderr_layer = fmt::layer().with_writer(std::io::stderr).with_ansi(true);
        tracing::subscriber::set_global_default(subscriber.with(stderr_layer))?;
    } else {
        tracing::subscriber::set_global_default(subscriber)?;
    }

    Ok(())
}
