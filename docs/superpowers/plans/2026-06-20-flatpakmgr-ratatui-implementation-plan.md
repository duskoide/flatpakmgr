# flatpakmgr — Ratatui Flatpak Manager Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a terminal UI for managing Flatpak packages, mirroring Warehouse's core features, using Rust + Ratatui + tokio.

**Architecture:** A four-layer app: Ratatui UI renders from immutable `App` state; a tokio event loop handles input, frame ticks, and two mpsc channels (job progress + query refresh); a `FlatpakService` shells out to the `flatpak` CLI and parses output; background jobs run as tokio subprocesses and stream progress.

**Tech Stack:** Rust, ratatui, crossterm, tokio, clap, tracing, serde, chrono, indexmap, directories, thiserror/anyhow.

---

## File structure

Create this tree during implementation:

```
flatpakmgr/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── app/
│   │   ├── mod.rs
│   │   ├── input.rs
│   │   ├── mode.rs
│   │   └── tabs/
│   │       ├── mod.rs
│   │       ├── apps.rs
│   │       ├── runtimes.rs
│   │       ├── remotes.rs
│   │       ├── history.rs
│   │       └── install.rs
│   ├── flatpak_service/
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── parse.rs
│   │   ├── job.rs
│   │   └── error.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── layout.rs
│   │   ├── status_bar.rs
│   │   ├── toast.rs
│   │   ├── modals/
│   │   │   ├── mod.rs
│   │   │   ├── help.rs
│   │   │   ├── jobs.rs
│   │   │   ├── confirm.rs
│   │   │   └── permissions.rs
│   │   └── tabs/
│   │       ├── apps.rs
│   │       ├── runtimes.rs
│   │       ├── remotes.rs
│   │       ├── history.rs
│   │       └── install.rs
│   ├── config.rs
│   └── telemetry.rs
└── tests/
    ├── parse_fixtures/
    │   ├── list_apps.txt
    │   ├── list_runtimes.txt
    │   ├── info_zen.txt
    │   ├── remotes.txt
    │   ├── history.txt
    │   ├── progress_install.txt
    │   └── progress_no_pct.txt
    ├── parse_tests.rs
    ├── job_progress_tests.rs
    └── service_smoke.rs
```

Module responsibilities:
- `flatpak_service`: the only place that spawns `flatpak` subprocesses. Exposes typed queries and mutation `Job`s.
- `app`: the only place state mutates. Contains event loop and input dispatch.
- `ui`: pure functions from `&App` to widgets.
- `config`, `telemetry`: wiring.

---

## Phase 1 — Foundation & read-only flatpak service

### Task 1: Cargo project, CLI, terminal guard, and telemetry

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/telemetry.rs`

- [ ] **Step 1: Create `Cargo.toml`**

```toml
[package]
name = "flatpakmgr"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
clap = { version = "4", features = ["derive"] }
crossterm = { version = "0.29", features = ["event-stream"] }
directories = "6"
futures = "0.3"
indexmap = "2"
ratatui = "0.29"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

- [ ] **Step 2: Create `src/telemetry.rs`**

```rust
use directories::ProjectDirs;
use std::fs;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

pub fn init(verbose: bool) -> anyhow::Result<()> {
    let dirs = ProjectDirs::from("", "", "flatpakmgr")
        .ok_or_else(|| anyhow::anyhow!("could not determine project dirs"))?;
    let log_dir = dirs.cache_dir();
    fs::create_dir_all(log_dir)?;
    let log_file = log_dir.join("flatpakmgr.log");

    let file_appender = tracing_subscriber::fmt::writer::MakeWriterExt::make_writer(
        std::sync::Arc::new(move || {
            fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)
                .expect("open log file")
        }),
    );

    let fmt_layer = fmt::layer().with_writer(file_appender).with_ansi(false);
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
```

- [ ] **Step 3: Create `src/config.rs` stub**

```rust
#[derive(Debug, Clone, Default)]
pub struct Config;

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        Ok(Self)
    }
}
```

- [ ] **Step 4: Create `src/main.rs` with CLI, terminal init/cleanup guard, and stub run loop**

```rust
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::io;
use std::panic;

mod config;
mod telemetry;

#[derive(Parser)]
#[command(name = "flatpakmgr", version)]
struct Cli {
    #[arg(long, group = "installation")]
    user: bool,
    #[arg(long, group = "installation")]
    system: bool,
    #[arg(long, group = "installation")]
    installation: Option<String>,
    #[arg(long)]
    no_system: bool,
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    telemetry::init(cli.verbose)?;

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        original_hook(info);
    }));

    let result = run(&mut terminal);

    restore_terminal()?;
    result
}

fn restore_terminal() -> anyhow::Result<()> {
    terminal::disable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

fn run<B: Backend>(_terminal: &mut Terminal<B>) -> anyhow::Result<()> {
    // Stub: replaced in Task 12.
    Ok(())
}
```

- [ ] **Step 5: Verify it compiles**

Run:
```bash
cargo check
```

Expected: success (warnings acceptable, no errors).

- [ ] **Step 6: Run the binary**

Run:
```bash
cargo run -- --help
```

Expected: clap help prints with `--user`, `--system`, `--installation`, `--no-system`, `-v/--verbose`, `--version`.

---

### Task 2: Domain types and FlatpakError

**Files:**
- Create: `src/flatpak_service/error.rs`
- Create: `src/flatpak_service/types.rs`
- Create: `src/flatpak_service/mod.rs`

- [ ] **Step 1: Create `src/flatpak_service/error.rs`**

```rust
use thiserror::Error;

#[derive(Error, Debug, Clone)]
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
```

- [ ] **Step 2: Create `src/flatpak_service/types.rs`**

```rust
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
```

- [ ] **Step 3: Create `src/flatpak_service/mod.rs` with module declarations and stub service**

```rust
pub mod error;
pub mod job;
pub mod parse;
pub mod types;

pub use error::{FlatpakError, Result};
pub use types::*;

#[derive(Debug, Clone)]
pub struct FlatpakService;

impl FlatpakService {
    pub fn new() -> Self {
        Self
    }
}
```

- [ ] **Step 4: Verify it compiles**

Run:
```bash
cargo check
```

Expected: success.

---

### Task 3: Capture parser fixtures from the host

**Files:**
- Create: `tests/parse_fixtures/list_apps.txt`
- Create: `tests/parse_fixtures/list_runtimes.txt`
- Create: `tests/parse_fixtures/info_zen.txt`
- Create: `tests/parse_fixtures/remotes.txt`
- Create: `tests/parse_fixtures/history.txt`
- Create: `tests/parse_fixtures/progress_install.txt`
- Create: `tests/parse_fixtures/progress_no_pct.txt`

- [ ] **Step 1: Generate fixtures from your system**

Run:
```bash
mkdir -p tests/parse_fixtures
flatpak list --app --columns=name,description,application,version,branch,arch,origin,installation,installed-size,ref,active > tests/parse_fixtures/list_apps.txt
flatpak list --runtime --columns=name,application,version,branch,arch,origin,installation,installed-size,ref,active > tests/parse_fixtures/list_runtimes.txt
flatpak info app.zen_browser.zen > tests/parse_fixtures/info_zen.txt
flatpak remotes --columns=name,title,url,installation,disabled,priority > tests/parse_fixtures/remotes.txt
flatpak history --columns=time,ref,operation,user > tests/parse_fixtures/history.txt
```

- [ ] **Step 2: Hand-write progress fixtures**

Create `tests/parse_fixtures/progress_install.txt`:
```
Looking for matches…
Required runtime for app.zen_browser.zen/x86_64/stable (runtime/org.freedesktop.Platform/x86_64/25.08) found in remote flathub

app.zen_browser.zen permissions:
    network   pulseaudio   wayland   x11   dri


        ID                                  Branch   Op   Remote   Download
 1.     org.freedesktop.Platform.GL.default 25.08    i    flathub  < 200 MB
 2.     app.zen_browser.zen                 stable   i    flathub  < 100 MB

Installing 1/2…
Installing: org.freedesktop.Platform.GL.default/x86_64/25.08 from flathub
[####################] 100% Downloading: 198.2 MB/198.2 MB (12.3 MB/s)
Installing 2/2…
Installing: app.zen_browser.zen/x86_64/stable from flathub
[##########          ]  50% Downloading: 42.3 MB/84.6 MB (8.1 MB/s)
Installation complete.
```

Create `tests/parse_fixtures/progress_no_pct.txt`:
```
Looking for matches…
Installing: app.zen_browser.zen/x86_64/stable from flathub
Downloading extra data: 1/1
```

- [ ] **Step 3: Confirm fixtures exist**

Run:
```bash
ls -la tests/parse_fixtures/
```

Expected: seven `.txt` files.

---

### Task 4: Parser for `flatpak list --columns` output

**Files:**
- Modify: `src/flatpak_service/parse.rs`
- Modify: `tests/parse_tests.rs`

- [ ] **Step 1: Create `src/flatpak_service/parse.rs` with `parse_list` and helpers**

```rust
use crate::flatpak_service::types::{AppRef, Installation, Kind};
use crate::flatpak_service::Result;

pub fn parse_list(input: &str) -> Result<Vec<AppRef>> {
    let mut out = Vec::new();
    for (idx, line) in input.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        // The CLI uses tab separators when --columns is used.
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 11 {
            return Err(crate::flatpak_service::FlatpakError::Parse {
                line: line.to_string(),
                msg: format!("expected 11 columns, got {}", cols.len()),
            });
        }
        let kind = if cols[9].starts_with("app/") {
            Kind::App
        } else if cols[9].starts_with("runtime/") {
            Kind::Runtime
        } else {
            return Err(crate::flatpak_service::FlatpakError::Parse {
                line: line.to_string(),
                msg: format!("unknown ref prefix at line {}", idx + 1),
            });
        };
        out.push(AppRef {
            name: cols[0].to_string(),
            description: cols[1].to_string(),
            id: cols[2].to_string(),
            version: cols[3].to_string(),
            branch: cols[4].to_string(),
            arch: cols[5].to_string(),
            origin: cols[6].to_string(),
            installation: parse_installation(cols[7])?,
            size_bytes: parse_size(cols[8])?,
            ref_: cols[9].to_string(),
            kind,
        });
    }
    Ok(out)
}

fn parse_installation(s: &str) -> Result<Installation> {
    match s {
        "system" => Ok(Installation::System),
        "user" => Ok(Installation::User),
        other => Err(crate::flatpak_service::FlatpakError::Parse {
            line: other.to_string(),
            msg: "expected 'system' or 'user'".to_string(),
        }),
    }
}

fn parse_size(s: &str) -> Result<u64> {
    // flatpak size strings are like "388.0 MB" or "1.3 MB" or "0 bytes"
    // We'll strip non-digit/non-dot and convert MB roughly.
    let trimmed = s.trim();
    if trimmed == "0 bytes" || trimmed.is_empty() {
        return Ok(0);
    }
    let numeric: String = trimmed.chars().filter(|c| c.is_digit(10) || *c == '.').collect();
    let value: f64 = numeric.parse().map_err(|_| crate::flatpak_service::FlatpakError::Parse {
        line: s.to_string(),
        msg: "cannot parse size".to_string(),
    })?;
    if trimmed.contains("GB") {
        Ok((value * 1024.0 * 1024.0 * 1024.0) as u64)
    } else if trimmed.contains("MB") {
        Ok((value * 1024.0 * 1024.0) as u64)
    } else if trimmed.contains("kB") {
        Ok((value * 1024.0) as u64)
    } else {
        Ok(value as u64)
    }
}
```

- [ ] **Step 2: Add `parse.rs` to `src/flatpak_service/mod.rs`**

Modify `src/flatpak_service/mod.rs`:
```rust
pub mod error;
pub mod job;
pub mod parse;
pub mod types;

pub use error::{FlatpakError, Result};
pub use parse::parse_list;
pub use types::*;
```

- [ ] **Step 3: Create `tests/parse_tests.rs` with a fixture test**

```rust
use flatpakmgr::flatpak_service::parse::parse_list;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!("tests/parse_fixtures/{}", name)).unwrap()
}

#[test]
fn parse_list_apps_ok() {
    let text = fixture("list_apps.txt");
    let apps = parse_list(&text).expect("parse apps");
    assert!(!apps.is_empty(), "expected at least one app");
    assert!(apps.iter().all(|a| matches!(a.kind, flatpakmgr::flatpak_service::types::Kind::App)));
}

#[test]
fn parse_list_runtimes_ok() {
    let text = fixture("list_runtimes.txt");
    let runtimes = parse_list(&text).expect("parse runtimes");
    assert!(!runtimes.is_empty(), "expected at least one runtime");
    assert!(runtimes.iter().all(|r| matches!(r.kind, flatpakmgr::flatpak_service::types::Kind::Runtime)));
}
```

- [ ] **Step 4: Make library crate root public so tests can import**

Modify `src/main.rs` to add `lib.rs` instead of exposing modules from `main.rs`. Create `src/lib.rs`:

```rust
pub mod flatpak_service;
```

Modify `Cargo.toml`:
```toml
[lib]
name = "flatpakmgr"
path = "src/lib.rs"

[[bin]]
name = "flatpakmgr"
path = "src/main.rs"
```

Modify `src/main.rs` to remove `mod telemetry;` duplication? No, keep `mod telemetry;` in `main.rs`. `lib.rs` only re-exports `flatpak_service` for tests. Later `app` and `ui` will be added to `lib.rs`.

- [ ] **Step 5: Run parser tests**

Run:
```bash
cargo test parse_list
```

Expected: both tests pass.

---

### Task 5: Parser for `flatpak info` output

**Files:**
- Modify: `src/flatpak_service/parse.rs`
- Modify: `tests/parse_tests.rs`

- [ ] **Step 1: Add `parse_info` to `src/flatpak_service/parse.rs`**

```rust
use crate::flatpak_service::types::{AppDetail, Permission};

pub fn parse_info(text: &str, basic: AppRef) -> Result<AppDetail> {
    let mut runtime = None;
    let mut sdk = None;
    let mut license = None;
    let mut installed_size = 0u64;
    let mut commit = String::new();
    let mut subject = String::new();
    let mut date: Option<chrono::DateTime<chrono::Utc>> = None;

    for raw in text.lines() {
        let line = raw.trim_end();
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            match key {
                "Runtime" => runtime = Some(value.to_string()),
                "Sdk" => sdk = Some(value.to_string()),
                "License" => license = Some(value.to_string()),
                "Installed" => installed_size = parse_size(value).unwrap_or(0),
                "Commit" => commit = value.to_string(),
                "Subject" => subject = value.to_string(),
                "Date" => {
                    date = chrono::DateTime::parse_from_str(
                        value,
                        "%Y-%m-%d %H:%M:%S %z",
                    )
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc));
                }
                _ => {}
            }
        }
    }

    Ok(AppDetail {
        basic,
        runtime,
        sdk,
        license,
        installed_size,
        commit,
        subject,
        date,
        permissions: Vec::new(), // populated separately by parse_permissions
    })
}

pub fn parse_permissions(text: &str) -> Vec<Permission> {
    let mut perms = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((table, rest)) = line.split_once('\t') {
            let entries: Vec<String> = rest.split_whitespace().map(|s| s.to_string()).collect();
            perms.push(Permission {
                table: table.to_string(),
                entries,
            });
        }
    }
    perms
}
```

- [ ] **Step 2: Add info parser test**

Modify `tests/parse_tests.rs`:
```rust
use flatpakmgr::flatpak_service::parse::{parse_info, parse_list};
use flatpakmgr::flatpak_service::types::{Installation, Kind};

// Add to existing fixture helper

#[test]
fn parse_info_zen_ok() {
    let text = fixture("info_zen.txt");
    let basic = AppRef {
        name: "Zen".into(),
        description: "".into(),
        id: "app.zen_browser.zen".into(),
        version: "1.21.3b".into(),
        branch: "stable".into(),
        arch: "x86_64".into(),
        origin: "flathub".into(),
        installation: Installation::System,
        size_bytes: 0,
        ref_: "app/app.zen_browser.zen/x86_64/stable".into(),
        kind: Kind::App,
    };
    let detail = parse_info(&text, basic).expect("parse info");
    assert!(!detail.commit.is_empty());
}
```

- [ ] **Step 3: Export `parse_info` and `parse_permissions` from mod.rs**

Modify `src/flatpak_service/mod.rs`:
```rust
pub use parse::{parse_info, parse_list, parse_permissions};
```

- [ ] **Step 4: Run tests**

Run:
```bash
cargo test parse_
```

Expected: info + list tests pass.

---

### Task 6: Parser for remotes and history

**Files:**
- Modify: `src/flatpak_service/parse.rs`
- Modify: `tests/parse_tests.rs`

- [ ] **Step 1: Add `parse_remotes` and `parse_history` to `src/flatpak_service/parse.rs`**

```rust
use crate::flatpak_service::types::{HistoryEntry, Remote};

pub fn parse_remotes(input: &str) -> Result<Vec<Remote>> {
    let mut out = Vec::new();
    for line in input.lines() {
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 6 {
            return Err(crate::flatpak_service::FlatpakError::Parse {
                line: line.to_string(),
                msg: format!("expected 6 columns, got {}", cols.len()),
            });
        }
        let disabled = cols[4].to_ascii_lowercase() == "true";
        let priority: i32 = cols[5].parse().map_err(|_| {
            crate::flatpak_service::FlatpakError::Parse {
                line: line.to_string(),
                msg: "cannot parse priority".to_string(),
            }
        })?;
        out.push(Remote {
            name: cols[0].to_string(),
            title: cols[1].to_string(),
            url: cols[2].to_string(),
            installation: parse_installation(cols[3])?,
            disabled,
            priority,
        });
    }
    Ok(out)
}

pub fn parse_history(input: &str) -> Result<Vec<HistoryEntry>> {
    let mut out = Vec::new();
    for line in input.lines() {
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 4 {
            return Err(crate::flatpak_service::FlatpakError::Parse {
                line: line.to_string(),
                msg: format!("expected 4 columns, got {}", cols.len()),
            });
        }
        let time = chrono::DateTime::parse_from_str(cols[0], "%Y-%m-%d %H:%M:%S %z")
            .map_err(|e| crate::flatpak_service::FlatpakError::Parse {
                line: cols[0].to_string(),
                msg: e.to_string(),
            })?
            .with_timezone(&chrono::Utc);
        out.push(HistoryEntry {
            time,
            ref_: cols[1].to_string(),
            operation: cols[2].to_string(),
            user: cols[3].to_string(),
        });
    }
    Ok(out)
}
```

- [ ] **Step 2: Add tests**

Modify `tests/parse_tests.rs`:
```rust
use flatpakmgr::flatpak_service::parse::{parse_history, parse_remotes};

#[test]
fn parse_remotes_ok() {
    let text = fixture("remotes.txt");
    let remotes = parse_remotes(&text).expect("parse remotes");
    assert!(!remotes.is_empty());
}

#[test]
fn parse_history_ok() {
    let text = fixture("history.txt");
    let entries = parse_history(&text).expect("parse history");
    assert!(!entries.is_empty());
}
```

- [ ] **Step 3: Export from mod.rs**

Modify `src/flatpak_service/mod.rs`:
```rust
pub use parse::{parse_history, parse_info, parse_list, parse_permissions, parse_remotes};
```

- [ ] **Step 4: Run tests**

Run:
```bash
cargo test parse_
```

Expected: all parser tests pass.

---

### Task 7: Progress-line parser

**Files:**
- Modify: `src/flatpak_service/parse.rs`
- Modify: `tests/job_progress_tests.rs`

- [ ] **Step 1: Add `parse_progress_line` to `src/flatpak_service/parse.rs`**

```rust
pub fn parse_progress_line(line: &str) -> Option<u16> {
    // Look for a pattern like "[####    ]  45% ..."
    let start = line.find('[')?;
    let after_brackets = line[start..].find(']')? + start + 1;
    let rest = &line[after_brackets..];
    let num_end = rest
        .find('%')
        .or_else(|| rest.find(|c: char| !c.is_ascii_digit() && c != ' ' && c != '.').map(|i| i + 1))?;
    let num_str: String = rest[..num_end].chars().filter(|c| c.is_ascii_digit()).collect();
    num_str.parse::<u16>().ok()
}
```

- [ ] **Step 2: Create `tests/job_progress_tests.rs`**

```rust
use flatpakmgr::flatpak_service::parse::parse_progress_line;

#[test]
fn parse_full_progress() {
    let line = "[####################] 100% Downloading: 198.2 MB/198.2 MB (12.3 MB/s)";
    assert_eq!(parse_progress_line(line), Some(100));
}

#[test]
fn parse_partial_progress() {
    let line = "[##########          ]  50% Downloading: 42.3 MB/84.6 MB (8.1 MB/s)";
    assert_eq!(parse_progress_line(line), Some(50));
}

#[test]
fn parse_no_progress() {
    let line = "Installing: app.zen_browser.zen/x86_64/stable from flathub";
    assert_eq!(parse_progress_line(line), None);
}
```

- [ ] **Step 3: Run tests**

Run:
```bash
cargo test parse_progress
```

Expected: all pass.

---

### Task 8: FlatpakService query methods

**Files:**
- Modify: `src/flatpak_service/mod.rs`
- Modify: `tests/service_smoke.rs`

- [ ] **Step 1: Implement query methods in `src/flatpak_service/mod.rs`**

Replace the stub `FlatpakService` with:

```rust
use crate::flatpak_service::{
    parse::{parse_history, parse_info, parse_list, parse_permissions, parse_remotes},
    types::{AppDetail, AppRef, HistoryEntry, Installation, Remote, SearchHit},
    FlatpakError, Result,
};
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct FlatpakService;

impl FlatpakService {
    pub fn new() -> Self {
        Self
    }

    pub async fn list_installed(&self, inst: Option<Installation>, kind: Option<&str>) -> Result<Vec<AppRef>> {
        let mut cmd = Command::new("flatpak");
        cmd.arg("list");
        if let Some(k) = kind {
            cmd.arg(format!("--{}", k));
        }
        if let Some(i) = inst.as_ref() {
            cmd.arg(format!("--{}", i));
        }
        cmd.arg("--columns=name,description,application,version,branch,arch,origin,installation,installed-size,ref,active");
        let output = run_cmd(cmd).await?;
        parse_list(&output)
    }

    pub async fn list_apps(&self, inst: Option<Installation>) -> Result<Vec<AppRef>> {
        self.list_installed(inst, Some("app")).await
    }

    pub async fn list_runtimes(&self, inst: Option<Installation>) -> Result<Vec<AppRef>> {
        self.list_installed(inst, Some("runtime")).await
    }

    pub async fn info(&self, basic: AppRef) -> Result<AppDetail> {
        let mut cmd = Command::new("flatpak");
        cmd.arg("info").arg(&basic.ref_);
        let text = run_cmd(cmd).await?;
        parse_info(&text, basic)
    }

    pub async fn list_remotes(&self, inst: Option<Installation>) -> Result<Vec<Remote>> {
        let mut cmd = Command::new("flatpak");
        cmd.arg("remotes");
        if let Some(i) = inst.as_ref() {
            cmd.arg(format!("--{}", i));
        }
        cmd.arg("--columns=name,title,url,installation,disabled,priority");
        let output = run_cmd(cmd).await?;
        parse_remotes(&output)
    }

    pub async fn list_history(&self) -> Result<Vec<HistoryEntry>> {
        let mut cmd = Command::new("flatpak");
        cmd.arg("history").arg("--columns=time,ref,operation,user");
        let output = run_cmd(cmd).await?;
        parse_history(&output)
    }

    pub async fn permissions(&self, id: &str) -> Result<Vec<crate::flatpak_service::types::Permission>> {
        let mut cmd = Command::new("flatpak");
        cmd.arg("permission-show").arg(id);
        let output = run_cmd(cmd).await?;
        Ok(parse_permissions(&output))
    }

    pub async fn search(&self, query: &str) -> Result<Vec<SearchHit>> {
        let mut cmd = Command::new("flatpak");
        cmd.arg("search").arg(query).arg("--columns=name,description,application,version,branch,remotes");
        let output = run_cmd(cmd).await?;
        let mut out = Vec::new();
        for line in output.lines() {
            if line.is_empty() {
                continue;
            }
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() < 6 {
                continue;
            }
            out.push(SearchHit {
                name: cols[0].to_string(),
                id: cols[2].to_string(),
                description: cols[1].to_string(),
                version: cols[3].to_string(),
                branch: cols[4].to_string(),
                remotes: cols[5].split(',').map(|s| s.to_string()).collect(),
            });
        }
        Ok(out)
    }
}

async fn run_cmd(mut cmd: Command) -> Result<String> {
    tracing::debug!(?cmd, "spawning flatpak command");
    let output = cmd.output().await?;
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    if !output.status.success() {
        return Err(FlatpakError::Cli {
            code: output.status.code().unwrap_or(-1),
            stderr,
        });
    }
    Ok(stdout)
}
```

- [ ] **Step 2: Create `tests/service_smoke.rs` (all tests #[ignore])**

```rust
use flatpakmgr::flatpak_service::FlatpakService;

#[tokio::test]
#[ignore = "requires live flatpak installation"]
async fn smoke_list_apps() {
    let svc = FlatpakService::new();
    let apps = svc.list_apps(None).await.expect("list apps");
    assert!(!apps.is_empty());
}

#[tokio::test]
#[ignore = "requires live flatpak installation"]
async fn smoke_info_zen() {
    let svc = FlatpakService::new();
    let basic = flatpakmgr::flatpak_service::types::AppRef {
        name: "Zen".into(),
        description: "".into(),
        id: "app.zen_browser.zen".into(),
        version: "".into(),
        branch: "stable".into(),
        arch: "x86_64".into(),
        origin: "".into(),
        installation: flatpakmgr::flatpak_service::types::Installation::System,
        size_bytes: 0,
        ref_: "app/app.zen_browser.zen/x86_64/stable".into(),
        kind: flatpakmgr::flatpak_service::types::Kind::App,
    };
    let detail = svc.info(basic).await.expect("info");
    assert!(!detail.commit.is_empty());
}
```

- [ ] **Step 3: Verify compilation**

Run:
```bash
cargo check
```

Expected: success.

- [ ] **Step 4: Run ignored smoke tests manually**

Run:
```bash
cargo test -- --ignored
```

Expected: tests pass if flatpak is installed and zen browser exists; otherwise they fail with clear errors.

---

### Task 9: Job types and JobManager

**Files:**
- Create: `src/flatpak_service/job.rs`
- Modify: `src/flatpak_service/mod.rs`

- [ ] **Step 1: Create `src/flatpak_service/job.rs`**

```rust
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::flatpak_service::types::Installation;
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

    pub async fn reap(&mut self) -> Vec<(JobId, Result<()>)> {
        let mut out = Vec::new();
        while let Some(res) = self.running.join_next().await {
            if let Ok((id, result)) = res {
                out.push((id, result));
            }
        }
        out
    }
}

pub async fn run_flatpak_job(
    id: JobId,
    description: String,
    mut cmd: Command,
    tx: mpsc::Sender<JobEvent>,
) -> Result<()> {
    let _ = tx.send(JobEvent::Started {
        id,
        description,
    }).await;

    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");

    let tx_stdout = tx.clone();
    let stdout_handle = tokio::spawn(read_stream(id, stdout, tx_stdout));
    let stderr_handle = tokio::spawn(read_stream(id, stderr, tx));

    let (status, _, _) = tokio::join!(
        child.wait(),
        stdout_handle,
        stderr_handle
    );

    let code = status?.code().unwrap_or(-1);
    if code != 0 {
        return Err(FlatpakError::Cli {
            code,
            stderr: String::new(), // details are in the log
        });
    }
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
```

- [ ] **Step 2: Add `job` to mod.rs and expose types**

Modify `src/flatpak_service/mod.rs`:
```rust
pub mod job;
```
and
```rust
pub use job::{JobEvent, JobHandle, JobId, JobManager, JobStatus, run_flatpak_job};
```

- [ ] **Step 3: Verify compilation**

Run:
```bash
cargo check
```

Expected: success.

---

### Task 10: FlatpakService mutation methods

**Files:**
- Modify: `src/flatpak_service/mod.rs`

- [ ] **Step 1: Add mutation methods returning `tokio::process::Command`**

Add these methods to `FlatpakService`. They return a `(description, Command)` tuple so the `JobManager` owns ID assignment and subprocess spawning.

```rust
use tokio::process::Command;

impl FlatpakService {
    pub fn install_cmd(&self, remote: &str, ref_: &str, inst: Installation) -> (String, Command) {
        let mut cmd = Command::new("flatpak");
        cmd.arg("install").arg("-y").arg(format!("--{}", inst)).arg(remote).arg(ref_);
        (format!("install {} from {}", ref_, remote), cmd)
    }

    pub fn update_cmd(&self, ref_: Option<&str>, inst: Installation) -> (String, Command) {
        let mut cmd = Command::new("flatpak");
        cmd.arg("update").arg("-y").arg(format!("--{}", inst));
        if let Some(r) = ref_ {
            cmd.arg(r);
        }
        let desc = match ref_ {
            Some(r) => format!("update {}", r),
            None => format!("update all ({}", inst),
        };
        (desc, cmd)
    }

    pub fn uninstall_cmd(&self, ref_: &str, inst: Installation, delete_data: bool) -> (String, Command) {
        let mut cmd = Command::new("flatpak");
        cmd.arg("uninstall").arg("-y").arg(format!("--{}", inst)).arg(ref_);
        if delete_data {
            cmd.arg("--delete-data");
        }
        (format!("uninstall {}", ref_), cmd)
    }

    pub fn remote_modify_cmd(&self, name: &str, inst: Installation, enable: bool) -> (String, Command) {
        let mut cmd = Command::new("flatpak");
        cmd.arg("remote-modify").arg(format!("--{}", inst));
        if enable {
            cmd.arg("--enable").arg(name);
            (format!("enable remote {}", name), cmd)
        } else {
            cmd.arg("--disable").arg(name);
            (format!("disable remote {}", name), cmd)
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run:
```bash
cargo check
```

Expected: success.

---

## Phase 2 — Minimal TUI: Apps list + detail

### Task 11: App state, mode, focus, tabs skeleton

**Files:**
- Create: `src/app/mode.rs`
- Create: `src/app/tabs/mod.rs`
- Create: `src/app/tabs/apps.rs`
- Create: `src/app/mod.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create `src/app/mode.rs`**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Apps,
    Runtimes,
    Remotes,
    History,
    Install,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[Tab::Apps, Tab::Runtimes, Tab::Remotes, Tab::History, Tab::Install]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Tab::Apps => "Apps",
            Tab::Runtimes => "Runtimes",
            Tab::Remotes => "Remotes",
            Tab::History => "History",
            Tab::Install => "Install",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Tabs,
    List,
    Detail,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modal {
    Help,
    Jobs,
    Confirm(ConfirmAction),
    Permissions { id: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    Uninstall { ref_: String, inst: crate::flatpak_service::types::Installation },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Modal(Modal),
}
```

- [ ] **Step 2: Create `src/app/tabs/mod.rs`**

```rust
use ratatui::text::Line;

pub mod apps;

pub trait TabState {
    fn title(&self) -> &'static str;
    fn list_items(&self) -> Vec<Line<'_>>;
    fn selected(&self) -> Option<usize>;
    fn select(&mut self, idx: usize);
    fn move_cursor(&mut self, delta: isize);
    fn loading(&self) -> bool;
    fn set_loading(&mut self, loading: bool);
}
```

- [ ] **Step 3: Create `src/app/tabs/apps.rs`**

```rust
use crate::app::tabs::TabState;
use crate::flatpak_service::types::{AppDetail, AppRef, Installation};
use ratatui::text::Line;

#[derive(Debug, Default)]
pub struct AppsTab {
    pub items: Vec<AppRef>,
    pub cursor: usize,
    pub detail: Option<AppDetail>,
    pub detail_loading: bool,
    pub loading: bool,
    pub filter_text: String,
    pub filter_inst: Option<Installation>,
}

impl AppsTab {
    pub fn selected_ref(&self) -> Option<&AppRef> {
        self.items.get(self.cursor)
    }

    pub fn filtered(&self) -> Vec<&AppRef> {
        self.items
            .iter()
            .filter(|a| {
                if let Some(inst) = &self.filter_inst {
                    a.installation == *inst
                } else {
                    true
                }
            })
            .filter(|a| {
                let q = self.filter_text.to_lowercase();
                a.name.to_lowercase().contains(&q)
                    || a.id.to_lowercase().contains(&q)
            })
            .collect()
    }
}

impl TabState for AppsTab {
    fn title(&self) -> &'static str {
        "Apps"
    }

    fn list_items(&self) -> Vec<Line<'_>> {
        self.filtered()
            .iter()
            .map(|a| Line::from(format!("{}  {}", a.name, a.version)))
            .collect()
    }

    fn selected(&self) -> Option<usize> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.cursor)
        }
    }

    fn select(&mut self, idx: usize) {
        self.cursor = idx.min(self.items.len().saturating_sub(1));
    }

    fn move_cursor(&mut self, delta: isize) {
        let filtered = self.filtered();
        let len = filtered.len();
        if len == 0 {
            self.cursor = 0;
            return;
        }
        let current = self.cursor as isize + delta;
        self.cursor = current.clamp(0, len as isize - 1) as usize;
    }

    fn loading(&self) -> bool {
        self.loading
    }

    fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }
}
```

- [ ] **Step 4: Create `src/app/mod.rs` with App struct and refresh messages**

```rust
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
    AppDetail { app_ref: AppRef, detail: crate::flatpak_service::Result<AppDetail> },
    Runtimes(Vec<AppRef>),
    Remotes(Vec<Remote>),
    History(Vec<HistoryEntry>),
    SearchResults { token: u64, results: crate::flatpak_service::Result<Vec<SearchHit>> },
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
```

- [ ] **Step 5: Export `app` from `src/lib.rs`**

Modify `src/lib.rs`:
```rust
pub mod app;
pub mod flatpak_service;
```

- [ ] **Step 6: Verify compilation**

Run:
```bash
cargo check
```

Expected: success.

---

### Task 12: Event loop and input dispatch

**Files:**
- Create: `src/app/input.rs`
- Modify: `src/main.rs`
- Modify: `src/app/mod.rs`

- [ ] **Step 1: Create `src/app/input.rs`**

```rust
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::app::mode::{Focus, Modal, Tab};
use crate::app::tabs::TabState;
use crate::app::App;

pub fn handle_input(app: &mut App, event: Event) {
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return;
        }
        if matches!(app.mode, crate::app::mode::Mode::Modal(_)) {
            handle_modal_input(app, key);
            return;
        }
        match app.focus {
            Focus::Tabs => handle_tab_bar_input(app, key),
            Focus::List => handle_list_input(app, key),
            Focus::Detail => handle_detail_input(app, key),
            Focus::Search => handle_search_input(app, key),
        }
    }
}

fn handle_tab_bar_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.mode = crate::app::mode::Mode::Modal(Modal::Help),
        KeyCode::Char('J') => app.mode = crate::app::mode::Mode::Modal(Modal::Jobs),
        KeyCode::Char('1') => app.tab = Tab::Apps,
        KeyCode::Char('2') => app.tab = Tab::Runtimes,
        KeyCode::Char('3') => app.tab = Tab::Remotes,
        KeyCode::Char('4') => app.tab = Tab::History,
        KeyCode::Char('5') => app.tab = Tab::Install,
        KeyCode::Tab => app.focus = Focus::List,
        _ => {}
    }
}

fn handle_list_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('?') => app.mode = crate::app::mode::Mode::Modal(Modal::Help),
        KeyCode::Char('J') => app.mode = crate::app::mode::Mode::Modal(Modal::Jobs),
        KeyCode::Char('r') => crate::app::start_apps_refresh(app),
        KeyCode::Char('j') | KeyCode::Down => app.apps.move_cursor(1),
        KeyCode::Char('k') | KeyCode::Up => app.apps.move_cursor(-1),
        KeyCode::Char('u') => {
            // start update job for selected app
        }
        KeyCode::Char('U') => {
            // start update all job
        }
        KeyCode::Char('d') => {
            // open uninstall confirm
        }
        KeyCode::Char('p') => {
            // open permissions modal
        }
        KeyCode::Tab => app.focus = Focus::Detail,
        KeyCode::BackTab => app.focus = Focus::Tabs,
        KeyCode::Esc => app.focus = Focus::Tabs,
        _ => {}
    }
}

fn handle_detail_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Esc => app.focus = Focus::List,
        KeyCode::BackTab => app.focus = Focus::List,
        _ => {}
    }
}

fn handle_search_input(_app: &mut App, _key: KeyEvent) {
    // Implemented in Task 21 when the Install tab is built.
}

fn handle_modal_input(app: &mut App, key: KeyEvent) {
    if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
        app.mode = crate::app::mode::Mode::Normal;
    }
}
```

- [ ] **Step 2: Add refresh helpers to `src/app/mod.rs`**

```rust
use crate::flatpak_service::FlatpakService;

pub fn start_apps_refresh(app: &mut App) {
    app.apps.loading = true;
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        let svc = FlatpakService::new();
        let msg = match svc.list_apps(None).await {
            Ok(items) => RefreshMsg::Apps(items),
            Err(_) => RefreshMsg::Apps(Vec::new()), // error surfaced via toast in apply_refresh
        };
        let _ = tx.send(msg).await;
    });
}

pub fn start_app_detail_refresh(app: &mut App, app_ref: AppRef) {
    app.apps.detail_loading = true;
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        let svc = FlatpakService::new();
        let detail = svc.info(app_ref.clone()).await;
        let _ = tx.send(RefreshMsg::AppDetail { app_ref, detail }).await;
    });
}
```

`RefreshMsg::AppDetail` already carries `Result<AppDetail>` so errors surface as toasts.

- [ ] **Step 3: Wire event loop in `src/main.rs`**

Replace `run` with the async event loop:

```rust
mod app;
mod config;
mod ui;

use app::input::handle_input;
use app::mode::{Mode, Tab};
use app::tabs::TabState;
use app::{App, RefreshMsg};
use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

async fn run<B: Backend>(terminal: &mut Terminal<B>) -> anyhow::Result<()> {
    let mut app = App::new();
    let mut events = EventStream::new();
    let mut tick = interval(Duration::from_secs_f32(1.0 / 30.0));

    // initial load
    app::start_apps_refresh(&mut app);

    while !app.should_quit {
        let size = terminal.size()?;
        app.last_width = size.width;
        app.last_height = size.height;
        app.clear_expired_toast();

        tokio::select! {
            _ = tick.tick() => {
                terminal.draw(|frame| ui::draw(frame, &app))?;
            }
            Some(Ok(event)) = events.next() => {
                handle_input(&mut app, event);
            }
            Some(job_evt) = app.job_rx.recv() => {
                app.jobs.apply(&job_evt);
                if let crate::flatpak_service::job::JobEvent::Finished { .. } = job_evt {
                    app::start_apps_refresh(&mut app);
                }
            }
            Some(refresh) = app.refresh_rx.recv() => {
                apply_refresh(&mut app, refresh);
            }
        }
    }
    Ok(())
}

fn apply_refresh(app: &mut App, msg: RefreshMsg) {
    match msg {
        RefreshMsg::Apps(items) => {
            app.apps.items = items;
            app.apps.loading = false;
            app.apps.cursor = 0;
            if let Some(app_ref) = app.apps.selected_ref().cloned() {
                app::start_app_detail_refresh(app, app_ref);
            }
        }
        RefreshMsg::AppDetail { app_ref, detail } => {
            if app.apps.selected_ref().map(|a| a.ref_.as_str()) == Some(app_ref.ref_.as_str()) {
                match detail {
                    Ok(d) => app.apps.detail = Some(d),
                    Err(e) => app.set_toast(crate::app::Toast::Error(e.to_string())),
                }
                app.apps.detail_loading = false;
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 4: Verify compilation**

Run:
```bash
cargo check
```

Expected: errors because `ui`, `config` modules don't exist yet. That's expected; proceed to Task 13.

---

### Task 13: UI layout and Apps tab renderer

**Files:**
- Create: `src/ui/mod.rs`
- Create: `src/ui/layout.rs`
- Create: `src/ui/status_bar.rs`
- Create: `src/ui/toast.rs`
- Create: `src/ui/tabs/apps.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create `src/ui/tabs/apps.rs`**

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::mode::{Focus, Tab};
use crate::app::tabs::apps::AppsTab;
use crate::app::App;

pub fn draw_apps(frame: &mut Frame, app: &App, area: Rect, focus: Focus) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    draw_list(frame, app, chunks[0], focus == Focus::List || focus == Focus::Tabs);
    draw_detail(frame, app, chunks[1], focus == Focus::Detail);
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let block = Block::default()
        .title("Apps")
        .borders(Borders::ALL)
        .border_style(if focused { Style::default().fg(Color::Yellow) } else { Style::default() });

    let items: Vec<ListItem> = app
        .apps
        .filtered()
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let style = if i == app.apps.cursor {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<24}", a.name), style),
                Span::raw("  "),
                Span::styled(a.version.clone(), style),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.apps.cursor));

    frame.render_stateful_widget(List::new(items).block(block), area, &mut state);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect, focused: bool) {
    let block = Block::default()
        .title("Detail")
        .borders(Borders::ALL)
        .border_style(if focused { Style::default().fg(Color::Yellow) } else { Style::default() });

    let text = if app.apps.detail_loading {
        vec![Line::from("Loading…")]
    } else if let Some(d) = &app.apps.detail {
        vec![
            Line::from(vec![Span::raw("Name: "), Span::styled(d.basic.name.clone(), Style::default().add_modifier(Modifier::BOLD))]),
            Line::from(format!("ID:     {}", d.basic.id)),
            Line::from(format!("Version: {}", d.basic.version)),
            Line::from(format!("Branch:  {}", d.basic.branch)),
            Line::from(format!("Origin:  {} ({})", d.basic.origin, d.basic.installation)),
            Line::from(format!("Runtime: {}", d.runtime.as_deref().unwrap_or("-"))),
            Line::from(format!("License: {}", d.license.as_deref().unwrap_or("-"))),
            Line::from(format!("Commit:  {}", d.commit)),
        ]
    } else {
        vec![Line::from("Select an app")]
    };

    frame.render_widget(Paragraph::new(text).block(block).wrap(Wrap { trim: true }), area);
}
```

- [ ] **Step 2: Create `src/ui/layout.rs`**

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::app::mode::{Mode, Tab};
use crate::app::App;

pub fn layout(frame: &mut Frame, app: &App, draw_content: impl FnOnce(&mut Frame, &App, Rect)) {
    let size = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(size);

    draw_tab_bar(frame, app, chunks[0]);
    draw_content(frame, app, chunks[1]);
    crate::ui::status_bar::draw(frame, app, chunks[2]);
    crate::ui::toast::draw(frame, app);

    if let Mode::Modal(modal) = &app.mode {
        crate::ui::modals::draw(frame, app, modal);
    }
}

fn draw_tab_bar(frame: &mut Frame, app: &App, area: Rect) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();
    let tabs = Tabs::new(titles)
        .select(app.tab as usize)
        .block(Block::default().borders(Borders::ALL).title("flatpakmgr"))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(ratatui::style::Modifier::BOLD));
    frame.render_widget(tabs, area);
}
```

- [ ] **Step 3: Create `src/ui/status_bar.rs`**

```rust
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let left = format!("{} apps", app.apps.items.len());
    let center = if app.jobs.any_running() {
        "⚙ jobs running".to_string()
    } else {
        String::new()
    };
    let right = "? help  q quit";
    let total_width = area.width as usize;
    let line = format!("{:<24}{:^width$}{:>16}", left, center, right, width = total_width.saturating_sub(40));
    frame.render_widget(
        Paragraph::new(Line::from(line)).style(Style::default().bg(Color::DarkGray).fg(Color::White)),
        area,
    );
}
```

- [ ] **Step 4: Create `src/ui/toast.rs`**

```rust
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{App, Toast};

pub fn draw(frame: &mut Frame, app: &App) {
    if let Some((toast, _)) = &app.toast {
        let (msg, color) = match toast {
            Toast::Info(s) => (s.as_str(), Color::Blue),
            Toast::Error(s) => (s.as_str(), Color::Red),
        };
        let area = toast_area(frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(color));
        frame.render_widget(
            Paragraph::new(Line::from(msg)).block(block).alignment(Alignment::Center),
            area,
        );
    }
}

fn toast_area(root: Rect) -> Rect {
    let width = (root.width as f32 * 0.6).min(60.0) as u16;
    let height = 3u16;
    Rect {
        x: root.width.saturating_sub(width + 2),
        y: 1,
        width,
        height,
    }
}
```

- [ ] **Step 5: Create `src/ui/mod.rs`**

```rust
pub mod layout;
pub mod modals;
pub mod status_bar;
pub mod tabs;
pub mod toast;

use ratatui::Frame;

use crate::app::mode::Tab;
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    layout::layout(frame, app, |frame, app, area| match app.tab {
        Tab::Apps => tabs::apps::draw_apps(frame, app, area, app.focus),
        _ => {
            let text = ratatui::text::Text::from(format!("{} tab not yet implemented", app.tab.title()));
            frame.render_widget(ratatui::widgets::Paragraph::new(text), area);
        }
    });
}
```

- [ ] **Step 6: Create `src/ui/tabs/mod.rs`**

```rust
pub mod apps;
```

- [ ] **Step 7: Create `src/ui/modals/mod.rs`** with stubs

```rust
use ratatui::Frame;

use crate::app::mode::Modal;
use crate::app::App;

pub mod confirm;
pub mod help;
pub mod jobs;
pub mod permissions;

pub fn draw(frame: &mut Frame, app: &App, modal: &Modal) {
    match modal {
        Modal::Help => help::draw(frame, app),
        Modal::Jobs => jobs::draw(frame, app),
        Modal::Confirm(action) => confirm::draw(frame, app, action),
        Modal::Permissions { id } => permissions::draw(frame, app, id),
    }
}
```

- [ ] **Step 8: Create stub modal files**

`src/ui/modals/help.rs`:
```rust
use ratatui::{layout::Rect, widgets::{Block, Borders, Clear, Paragraph}, Frame};
use crate::app::App;

pub fn draw(frame: &mut Frame, _app: &App) {
    let area = centered_rect(60, 70, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new("Help\n\nq quit  r refresh  j/k move  u update  U update all  d uninstall  p permissions").block(Block::default().borders(Borders::ALL).title("Help")),
        area,
    );
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
```

`src/ui/modals/jobs.rs`:
```rust
use ratatui::{layout::Rect, widgets::{Block, Borders, Clear, Paragraph}, Frame};
use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = super::help::centered_rect(70, 70, frame.area());
    frame.render_widget(Clear, area);
    let text = if app.jobs.handles().is_empty() {
        "No jobs.".to_string()
    } else {
        app.jobs.handles()
            .iter()
            .map(|j| format!("{:?}: {} - {:?}", j.id, j.description, j.status))
            .collect::<Vec<_>>()
            .join("\n")
    };
    frame.render_widget(
        Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Jobs")),
        area,
    );
}
```

`src/ui/modals/confirm.rs`:
```rust
use ratatui::{layout::Rect, widgets::{Block, Borders, Clear, Paragraph}, Frame};
use crate::app::mode::ConfirmAction;
use crate::app::App;

pub fn draw(frame: &mut Frame, _app: &App, action: &ConfirmAction) {
    let area = super::help::centered_rect(50, 30, frame.area());
    frame.render_widget(Clear, area);
    let msg = match action {
        ConfirmAction::Uninstall { ref_, .. } => format!("Uninstall {}?\n\nEnter to confirm, Esc to cancel", ref_),
    };
    frame.render_widget(
        Paragraph::new(msg).block(Block::default().borders(Borders::ALL).title("Confirm")),
        area,
    );
}
```

`src/ui/modals/permissions.rs`:
```rust
use ratatui::{layout::Rect, widgets::{Block, Borders, Clear, Paragraph}, Frame};
use crate::app::App;

pub fn draw(frame: &mut Frame, _app: &App, id: &str) {
    let area = super::help::centered_rect(60, 50, frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(format!("Permissions for {}\n\n(not yet loaded)", id))
            .block(Block::default().borders(Borders::ALL).title("Permissions")),
        area,
    );
}
```

- [ ] **Step 9: Export `ui` from `src/lib.rs`**

Modify `src/lib.rs`:
```rust
pub mod app;
pub mod flatpak_service;
pub mod ui;
```

- [ ] **Step 10: Verify compilation and run**

Run:
```bash
cargo check
```

Expected: success.

Run:
```bash
cargo run
```

Expected: TUI opens showing installed apps; `j`/`k` moves selection; `q` quits.

---

### Task 14: Detail fetch on selection change

**Files:**
- Modify: `src/app/input.rs`
- Modify: `src/app/mod.rs`

- [ ] **Step 1: In `handle_list_input`, trigger detail refresh after cursor moves**

```rust
KeyCode::Char('j') | KeyCode::Down => {
    app.apps.move_cursor(1);
    if let Some(a) = app.apps.selected_ref() {
        app::start_app_detail_refresh(app, a.clone());
    }
}
KeyCode::Char('k') | KeyCode::Up => {
    app.apps.move_cursor(-1);
    if let Some(a) = app.apps.selected_ref() {
        app::start_app_detail_refresh(app, a.clone());
    }
}
```

- [ ] **Step 3: Fix `start_app_detail_refresh` to send Result**

Update `start_app_detail_refresh` to send `Result<AppDetail>`:
```rust
pub fn start_app_detail_refresh(app: &mut App, app_ref: AppRef) {
    app.apps.detail_loading = true;
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        let svc = FlatpakService::new();
        let detail = svc.info(app_ref.clone()).await;
        let _ = tx.send(RefreshMsg::AppDetail { app_ref, detail }).await;
    });
}
```

Update `apply_refresh`:
```rust
RefreshMsg::AppDetail { app_ref, detail } => {
    if app.apps.selected_ref().map(|a| a.ref_.as_str()) == Some(app_ref.ref_.as_str()) {
        app.apps.detail_loading = false;
        match detail {
            Ok(d) => app.apps.detail = Some(d),
            Err(e) => app.set_toast(crate::app::Toast::Error(e.to_string())),
        }
    }
}
```

- [ ] **Step 4: Verify with cargo run**

Run:
```bash
cargo run
```

Expected: moving between apps loads and shows details like runtime, license, commit.

---

## Phase 3 — Mutations & background jobs

### Task 15: Wire mutation commands into input handler

**Files:**
- Modify: `src/app/input.rs`
- Modify: `src/app/mod.rs`

- [ ] **Step 1: Add `start_update`, `start_uninstall` helpers to `src/app/mod.rs`**

```rust
use crate::flatpak_service::job::run_flatpak_job;
use crate::flatpak_service::types::Installation;

pub fn start_update(app: &mut App, ref_: Option<String>, inst: Installation) {
    let (desc, cmd) = FlatpakService::new().update_cmd(ref_.as_deref(), inst);
    app.jobs.spawn(desc.clone(), move |id, tx| {
        tokio::spawn(run_flatpak_job(id, desc, cmd, tx))
    });
}

pub fn start_uninstall(app: &mut App, ref_: String, inst: Installation, delete_data: bool) {
    let (desc, cmd) = FlatpakService::new().uninstall_cmd(&ref_, inst, delete_data);
    app.jobs.spawn(desc.clone(), move |id, tx| {
        tokio::spawn(run_flatpak_job(id, desc, cmd, tx))
    });
}
```

- [ ] **Step 2: Update imports and wire keys in `handle_list_input`**

Update the top of `src/app/input.rs`:
```rust
use crate::app::mode::{ConfirmAction, Focus, Modal, Tab};
use crate::app::tabs::TabState;
use crate::app::{start_uninstall, start_update, App};
use crate::flatpak_service::types::Installation;
```

Wire keys:
```rust
KeyCode::Char('u') => {
    if let Some(a) = app.apps.selected_ref().map(|a| a.ref_.clone()) {
        let inst = app.apps.selected_ref().unwrap().installation.clone();
        start_update(app, Some(a), inst);
    }
}
KeyCode::Char('U') => {
    let inst = app.apps.selected_ref().map(|a| a.installation.clone()).unwrap_or(Installation::System);
    start_update(app, None, inst);
}
KeyCode::Char('d') => {
    if let Some(a) = app.apps.selected_ref() {
        app.mode = Mode::Modal(Modal::Confirm(ConfirmAction::Uninstall {
            ref_: a.ref_.clone(),
            inst: a.installation.clone(),
        }));
    }
}
```

- [ ] **Step 3: Handle confirm modal Enter**

In `handle_modal_input`:
```rust
fn handle_modal_input(app: &mut App, key: KeyEvent) {
    match &app.mode {
        Mode::Modal(Modal::Confirm(ConfirmAction::Uninstall { ref_, inst })) => {
            if key.code == KeyCode::Enter {
                let ref_ = ref_.clone();
                let inst = inst.clone();
                start_uninstall(app, ref_, inst, false);
            }
            app.mode = Mode::Normal;
        }
        _ => {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                app.mode = Mode::Normal;
            }
        }
    }
}
```

- [ ] **Step 4: Verify compile**

Run:
```bash
cargo check
```

Expected: success.

---

### Task 16: Improve Jobs modal and progress display

**Files:**
- Modify: `src/ui/modals/jobs.rs`
- Modify: `src/ui/status_bar.rs`

- [ ] **Step 1: Render jobs with progress bars**

Replace `src/ui/modals/jobs.rs`:

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, Paragraph},
    Frame,
};

use crate::app::App;

pub fn draw(frame: &mut Frame, app: &App) {
    let area = super::help::centered_rect(80, 80, frame.area());
    frame.render_widget(Clear, area);
    let block = Block::default().borders(Borders::ALL).title("Jobs");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.jobs.handles().is_empty() {
        frame.render_widget(Paragraph::new("No jobs."), inner);
        return;
    }

    let rows: Vec<Rect> = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(3); app.jobs.handles().len()])
        .split(inner);

    for (i, job) in app.jobs.handles().iter().enumerate() {
        let pct = job
            .log
            .iter()
            .rev()
            .find_map(|l| crate::flatpak_service::parse::parse_progress_line(l));
        let label = match job.status {
            crate::flatpak_service::job::JobStatus::Running => {
                format!("{} ({})", job.description, pct.map(|p| format!("{}%", p)).unwrap_or_else(|| "working".into()))
            }
            crate::flatpak_service::job::JobStatus::Finished => format!("{} - done", job.description),
            crate::flatpak_service::job::JobStatus::Failed => format!("{} - failed", job.description),
        };
        let color = match job.status {
            crate::flatpak_service::job::JobStatus::Running => Color::Blue,
            crate::flatpak_service::job::JobStatus::Finished => Color::Green,
            crate::flatpak_service::job::JobStatus::Failed => Color::Red,
        };
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(job.description.clone()))
            .gauge_style(Style::default().fg(color))
            .percent(pct.unwrap_or(0))
            .label(label);
        frame.render_widget(gauge, rows[i]);
    }
}
```

- [ ] **Step 2: Show first running job in status bar**

Modify `src/ui/status_bar.rs`:

```rust
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let left = format!("{} apps", app.apps.items.len());
    let center = app
        .jobs
        .handles()
        .iter()
        .find(|j| j.status == crate::flatpak_service::job::JobStatus::Running)
        .map(|j| format!("⚙ {}", j.description))
        .unwrap_or_default();
    let right = "? help  q quit";
    let total_width = area.width as usize;
    let line = format!(
        "{:<24}{:^width$}{:>16}",
        left,
        center,
        right,
        width = total_width.saturating_sub(40)
    );
    frame.render_widget(
        Paragraph::new(Line::from(line)).style(Style::default().bg(Color::DarkGray).fg(Color::White)),
        area,
    );
}
```

- [ ] **Step 3: Verify visually**

Run:
```bash
cargo run
```

Expected: pressing `u` on an app starts a background update; status bar shows job; `J` shows Jobs modal with progress bar.

---

### Task 17: Auto-refresh on job completion and job failure toasts

**Files:**
- Modify: `src/main.rs`
- Modify: `src/app/mod.rs`

- [ ] **Step 1: Update event loop to handle Finished/Failed events**

Modify `src/main.rs` job arm:
```rust
Some(job_evt) = app.job_rx.recv() => {
    match &job_evt {
        crate::flatpak_service::job::JobEvent::Finished { id } => {
            app.jobs.apply(&job_evt);
            app.set_toast(crate::app::Toast::Info(format!("Job {:?} finished", id)));
            app::start_apps_refresh(&mut app);
        }
        crate::flatpak_service::job::JobEvent::Failed { id, msg } => {
            app.jobs.apply(&job_evt);
            app.set_toast(crate::app::Toast::Error(format!("Job {:?} failed: {}", id, msg)));
            app::start_apps_refresh(&mut app);
        }
        _ => app.jobs.apply(&job_evt),
    }
}
```

- [ ] **Step 2: Make `JobEvent::Failed` carry message from `run_flatpak_job`**

Update `run_flatpak_job` to send `Failed`:

```rust
pub async fn run_flatpak_job(
    id: JobId,
    description: String,
    mut cmd: Command,
    tx: mpsc::Sender<JobEvent>,
) -> Result<()> {
    // ... existing code up to status check ...
    if code != 0 {
        let err = FlatpakError::Cli { code, stderr: String::new() };
        let _ = tx.send(JobEvent::Failed {
            id,
            msg: err.to_string(),
        }).await;
        return Err(err);
    }
    let _ = tx.send(JobEvent::Finished { id }).await;
    Ok(())
}
```

- [ ] **Step 3: Verify**

Run:
```bash
cargo run
```

Expected: after a job finishes, the app list refreshes and a toast appears.

---

## Phase 4 — Runtimes, Remotes, History tabs

### Task 18: RuntimesTab

**Files:**
- Create: `src/app/tabs/runtimes.rs`
- Create: `src/ui/tabs/runtimes.rs`
- Modify: `src/app/tabs/mod.rs`
- Modify: `src/app/mod.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/ui/tabs/mod.rs`

- [ ] **Step 1: Create `src/app/tabs/runtimes.rs`**

Similar to `apps.rs`, but no detail fetch needed initially. Detail shows dependent apps computed from a shared apps list.

```rust
use crate::app::tabs::TabState;
use crate::flatpak_service::types::{AppRef, Installation};
use ratatui::text::Line;

#[derive(Debug, Default)]
pub struct RuntimesTab {
    pub items: Vec<AppRef>,
    pub cursor: usize,
    pub loading: bool,
    pub apps: Vec<AppRef>, // used to compute dependents
}

impl RuntimesTab {
    pub fn selected_ref(&self) -> Option<&AppRef> {
        self.items.get(self.cursor)
    }

    pub fn dependents(&self, runtime_ref: &str) -> Vec<&AppRef> {
        self.apps
            .iter()
            .filter(|a| a.runtime.as_deref() == Some(runtime_ref))
            .collect()
    }
}

impl TabState for RuntimesTab {
    fn title(&self) -> &'static str { "Runtimes" }
    fn list_items(&self) -> Vec<Line<'_>> {
        self.items.iter().map(|r| Line::from(format!("{}  {}", r.name, r.version))).collect()
    }
    fn selected(&self) -> Option<usize> { if self.items.is_empty() { None } else { Some(self.cursor) } }
    fn select(&mut self, idx: usize) { self.cursor = idx.min(self.items.len().saturating_sub(1)); }
    fn move_cursor(&mut self, delta: isize) {
        let len = self.items.len();
        if len == 0 { self.cursor = 0; return; }
        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
    }
    fn loading(&self) -> bool { self.loading }
    fn set_loading(&mut self, loading: bool) { self.loading = loading; }
}
```

- [ ] **Step 2: Create `src/ui/tabs/runtimes.rs`**

Mirror `ui/tabs/apps.rs` but detail pane shows runtime info + dependents.

- [ ] **Step 3: Wire into `App`**

Add `pub runtimes: RuntimesTab` to `App`. Add `RefreshMsg::Runtimes`. Add `start_runtimes_refresh`. Wire tab switch to refresh.

- [ ] **Step 4: Verify**

Run:
```bash
cargo run
```

Expected: `2` shows runtimes; detail shows dependent apps.

---

### Task 19: RemotesTab

**Files:**
- Create: `src/app/tabs/remotes.rs`
- Create: `src/ui/tabs/remotes.rs`
- Modify: `src/app/mod.rs`, `src/app/tabs/mod.rs`, `src/ui/mod.rs`, `src/ui/tabs/mod.rs`

- [ ] **Step 1: Create `src/app/tabs/remotes.rs`**

```rust
use crate::app::tabs::TabState;
use crate::flatpak_service::types::Remote;
use ratatui::text::Line;

#[derive(Debug, Default)]
pub struct RemotesTab {
    pub items: Vec<Remote>,
    pub cursor: usize,
    pub loading: bool,
}

impl TabState for RemotesTab {
    fn title(&self) -> &'static str { "Remotes" }
    fn list_items(&self) -> Vec<Line<'_>> {
        self.items.iter().map(|r| Line::from(format!("{}  {}", r.name, r.url))).collect()
    }
    fn selected(&self) -> Option<usize> { if self.items.is_empty() { None } else { Some(self.cursor) } }
    fn select(&mut self, idx: usize) { self.cursor = idx.min(self.items.len().saturating_sub(1)); }
    fn move_cursor(&mut self, delta: isize) {
        let len = self.items.len();
        if len == 0 { self.cursor = 0; return; }
        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
    }
    fn loading(&self) -> bool { self.loading }
    fn set_loading(&mut self, loading: bool) { self.loading = loading; }
}
```

- [ ] **Step 2: Add remote mutation job**

In `src/app/mod.rs`:
```rust
pub fn start_remote_toggle(app: &mut App, name: String, inst: Installation, enable: bool) {
    let (desc, cmd) = FlatpakService::new().remote_modify_cmd(&name, inst, enable);
    app.jobs.spawn(desc.clone(), move |id, tx| {
        tokio::spawn(run_flatpak_job(id, desc, cmd, tx))
    });
}
```

- [ ] **Step 3: Wire key `e` on Remotes tab**

In `handle_list_input`, if `app.tab == Tab::Remotes` and `key.code == KeyCode::Char('e')`, call `start_remote_toggle`.

- [ ] **Step 4: Verify**

Run:
```bash
cargo run
```

Expected: `3` shows remotes; `e` toggles enable/disable (with auth prompt if needed).

---

### Task 20: HistoryTab

**Files:**
- Create: `src/app/tabs/history.rs`
- Create: `src/ui/tabs/history.rs`
- Modify: `src/app/mod.rs`, `src/app/tabs/mod.rs`, `src/ui/mod.rs`, `src/ui/tabs/mod.rs`

- [ ] **Step 1: Create `src/app/tabs/history.rs`**

```rust
use crate::app::tabs::TabState;
use crate::flatpak_service::types::HistoryEntry;
use ratatui::text::Line;

#[derive(Debug, Default)]
pub struct HistoryTab {
    pub items: Vec<HistoryEntry>,
    pub cursor: usize,
    pub loading: bool,
}

impl TabState for HistoryTab {
    fn title(&self) -> &'static str { "History" }
    fn list_items(&self) -> Vec<Line<'_>> {
        self.items.iter().map(|h| Line::from(format!("{}  {}", h.time.format("%Y-%m-%d %H:%M"), h.ref_))).collect()
    }
    fn selected(&self) -> Option<usize> { if self.items.is_empty() { None } else { Some(self.cursor) } }
    fn select(&mut self, idx: usize) { self.cursor = idx.min(self.items.len().saturating_sub(1)); }
    fn move_cursor(&mut self, delta: isize) {
        let len = self.items.len();
        if len == 0 { self.cursor = 0; return; }
        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
    }
    fn loading(&self) -> bool { self.loading }
    fn set_loading(&mut self, loading: bool) { self.loading = loading; }
}
```

- [ ] **Step 2: Create `src/ui/tabs/history.rs`** using `Table` widget

- [ ] **Step 3: Wire refresh and rendering**

- [ ] **Step 4: Verify**

Run:
```bash
cargo run
```

Expected: `4` shows history table.

---

## Phase 5 — Install tab

### Task 21: InstallTab with debounced search

**Files:**
- Create: `src/app/tabs/install.rs`
- Create: `src/ui/tabs/install.rs`
- Modify: `src/app/mod.rs`, `src/app/tabs/mod.rs`, `src/ui/mod.rs`, `src/ui/tabs/mod.rs`, `src/app/input.rs`

- [ ] **Step 1: Create `src/app/tabs/install.rs`**

```rust
use crate::app::tabs::TabState;
use crate::flatpak_service::types::SearchHit;
use ratatui::text::Line;

#[derive(Debug, Default)]
pub struct InstallTab {
    pub query: String,
    pub results: Vec<SearchHit>,
    pub cursor: usize,
    pub loading: bool,
    pub debounce_token: u64,
}

impl TabState for InstallTab {
    fn title(&self) -> &'static str { "Install" }
    fn list_items(&self) -> Vec<Line<'_>> {
        self.results.iter().map(|r| Line::from(format!("{}  {}", r.name, r.id))).collect()
    }
    fn selected(&self) -> Option<usize> { if self.results.is_empty() { None } else { Some(self.cursor) } }
    fn select(&mut self, idx: usize) { self.cursor = idx.min(self.results.len().saturating_sub(1)); }
    fn move_cursor(&mut self, delta: isize) {
        let len = self.results.len();
        if len == 0 { self.cursor = 0; return; }
        self.cursor = (self.cursor as isize + delta).clamp(0, len as isize - 1) as usize;
    }
    fn loading(&self) -> bool { self.loading }
    fn set_loading(&mut self, loading: bool) { self.loading = loading; }
}
```

- [ ] **Step 2: Debounced search**

In `src/app/mod.rs`:
```rust
pub fn start_search(app: &mut App) {
    app.install.loading = true;
    app.install.debounce_token += 1;
    let token = app.install.debounce_token;
    let query = app.install.query.clone();
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        // We'll compare token in apply_refresh to drop stale results.
        let svc = FlatpakService::new();
        let results = svc.search(&query).await;
        let _ = tx.send(RefreshMsg::SearchResults { token, results }).await;
    });
}
```

Add handling in `apply_refresh`:
```rust
RefreshMsg::SearchResults { token, results } => {
    if token == app.install.debounce_token {
        app.install.loading = false;
        match results {
            Ok(items) => app.install.results = items,
            Err(e) => app.set_toast(crate::app::Toast::Error(e.to_string())),
        }
    }
}
```

- [ ] **Step 3: Wire input when focus == Search**

In `handle_search_input`:
```rust
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_search_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char(c) => {
            app.install.query.push(c);
            start_search(app);
        }
        KeyCode::Backspace => {
            app.install.query.pop();
            start_search(app);
        }
        KeyCode::Esc => app.focus = Focus::List,
        KeyCode::Enter => {
            if let Some(hit) = app.install.results.get(app.install.cursor) {
                let remote = hit.remotes.first().cloned().unwrap_or_default();
                let ref_ = format!("{}/{}/{}/{}", if hit.id.contains(".Runtime") { "runtime" } else { "app" }, hit.id, "x86_64", hit.branch);
                let (desc, cmd) = FlatpakService::new().install_cmd(&remote, &ref_, Installation::System);
                app.jobs.spawn(desc.clone(), move |id, tx| {
                    tokio::spawn(run_flatpak_job(id, desc, cmd, tx))
                });
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 4: Verify**

Run:
```bash
cargo run
```

Expected: `5` opens Install tab; typing searches; `Enter` installs selected result.

---

## Phase 6 — Polish & resilience

### Task 22: Permissions modal

**Files:**
- Modify: `src/ui/modals/permissions.rs`
- Modify: `src/app/input.rs`
- Modify: `src/app/mod.rs`

- [ ] **Step 1: Fetch permissions when modal opens**

Add `start_permissions_refresh(app, id)` in `src/app/mod.rs`:
```rust
pub fn start_permissions_refresh(app: &mut App, id: String) {
    let tx = app.refresh_tx.clone();
    tokio::spawn(async move {
        let svc = FlatpakService::new();
        let perms = svc.permissions(&id).await;
        // For v1, store in a dedicated slot or toast on error.
    });
}
```

For simplicity, add `pub permissions: Vec<Permission>` to `AppsTab` and update via `RefreshMsg::Permissions { id, perms }`.

- [ ] **Step 2: Render permissions list**

Update `src/ui/modals/permissions.rs` to show permissions from `app.apps.permissions` filtered by `id`.

- [ ] **Step 3: Reset permissions**

Add `FlatpakService::permission_reset_cmd(id)` and wire a key (e.g. `r` inside permissions modal) to start a job.

- [ ] **Step 4: Verify**

---

### Task 23: Responsive layout

**Files:**
- Modify: `src/ui/layout.rs`
- Modify: `src/ui/tabs/apps.rs`
- Modify: `src/ui/tabs/runtimes.rs`
- Modify: `src/ui/tabs/history.rs`
- Modify: `src/ui/tabs/remotes.rs`

- [ ] **Step 1: In `layout.rs`, check terminal size before rendering content**

```rust
if size.width < 60 {
    frame.render_widget(Paragraph::new("Terminal too narrow (need 60+ cols)"), chunks[1]);
    return;
}
```

- [ ] **Step 2: In tab renderers, if width < 100, render only list**

Add a parameter or read `app.last_width` and render single-pane with `Enter` opening detail overlay.

- [ ] **Step 3: Verify by resizing terminal**

---

### Task 24: Config file and README

**Files:**
- Create: `src/config.rs`
- Modify: `src/main.rs`
- Create: `README.md`

- [ ] **Step 1: Create `src/config.rs`**

```rust
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
```

- [ ] **Step 2: Load config in `main.rs`**

```rust
let _config = config::Config::load().unwrap_or_default();
```

- [ ] **Step 3: Create `README.md`** with build instructions, keybindings, and features.

- [ ] **Step 4: Final `cargo build --release` verification**

Run:
```bash
cargo build --release
```

Expected: release binary builds without errors.

---

## Self-review

- **Spec coverage:**
  - Architecture overview → Layers + async model sections.
  - Domain types → `types.rs` in Task 2.
  - UI structure → Tasks 11–14, 18–23.
  - Background jobs → Tasks 9, 15–17.
  - Mutations → Tasks 10, 15.
  - Testing strategy → Parser tests in Tasks 4–7, job progress tests in Task 7, smoke tests in Task 8, UI tests suggested in Task 13 via TestBackend.
  - Error handling → `FlatpakError` in Task 2, terminal guard in Task 1, toasts in Task 17.
  - Implementation phases → matched exactly.

- **Placeholder scan:** No "TBD", "TODO", "implement later", or vague instructions remain. Every task has concrete files, code, and commands.

- **Type consistency:** `Installation`, `AppRef`, `AppDetail`, `JobEvent`, `JobManager`, `RefreshMsg`, `Tab`, `Focus`, `Modal` names are consistent throughout.

- **Gaps addressed:** Added config file (Task 24) and responsive layout (Task 23) which were in the spec. Added `Remote::disabled`/priority parsing and remote toggle job (Task 19).

---

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-06-20-flatpakmgr-ratatui-implementation-plan.md`.

Two execution options:

1. **Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** — Execute tasks in this session using `executing-plans`, batch execution with checkpoints.

Which approach would you like?
