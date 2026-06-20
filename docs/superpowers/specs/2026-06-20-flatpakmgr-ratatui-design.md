# flatpakmgr — Ratatui Flatpak Manager

## Status

Design spec. Approved sections:
- Architecture overview
- Data model & Flatpak interface
- UI structure & navigation
- Application state & event flow
- Project layout, testing & error strategy
- Implementation phases

## Goal

Build a terminal-based Flatpak package manager inspired by Warehouse, written in Rust with Ratatui. It will let users browse installed applications/runtimes, manage remotes, view history, search and install from remotes, and run common mutations (update, uninstall) with live background progress.

## Scope (v1)

Included:
- Browse installed **applications** and **runtimes** with list + detail panes.
- View app details: version, ref, runtime, size, license, commit, subject, date, permissions.
- Browse **remotes** with enable/disable.
- View **history** as a scrollable table.
- **Search remotes** and install results.
- Mutations: update one, update all, uninstall, install from search.
- Background jobs with status bar and dedicated Jobs modal.
- Permissions view/reset for an app.

Excluded from v1:
- Snapshots / rollback.
- Batch selection and batch operations.
- Orphan/runtime cleanup wizard.
- App data backup/restore.
- Flatpak config editor beyond remote enable/disable.
- Flatpak repair / create-usb.

## Architectural decision: shell out to the `flatpak` CLI

We will use the `flatpak` CLI as the integration surface. Listing commands will use the `--columns=...` option for machine-readable output. Mutations will be `tokio::process::Command` subprocesses whose stderr is streamed and parsed for progress.

Alternatives considered:
- libflatpak bindings — rejected due to heavy C/glib dependency, binding maturity, and awkward integration with tokio.
- Hybrid CLI+library — rejected because it added complexity without a clear benefit.

The CLI is Flatpak's stable public interface, and flatpak itself invokes polkit for privileged system operations, so authentication is handled for us.

## Async model

Runtime: `tokio` multi-threaded (`tokio = { version = "1", features = ["full"] }`).

Event loop shape (one `tokio::select!`):

1. Frame tick at ~30 FPS drives `terminal.draw(...)`.
2. `crossterm::event::EventStream` yields user input.
3. `mpsc::Receiver<JobEvent>` yields progress/state from background mutations.
4. `mpsc::Receiver<RefreshMsg>` yields completed query results (installed list, app info, history, search).

No shared locks on the hot path: the event loop is the only owner of `App`. Background tasks communicate exclusively via channels.

## Layers

```
┌─────────────────────────────────────────────────┐
│  UI layer (ratatui widgets, App state machine)  │
│   - draws from App::state, never calls flatpak   │
└───────────────▲───────────────────┬─────────────┘
        events │           commands │
┌───────────────┴───────────────────▼─────────────┐
│  App controller (event loop, tokio::select!)     │
│   - input events, frame ticks, mpsc progress      │
└───────────────▲───────────────────┬─────────────┘
        queries│           job cmds │
┌───────────────┴───────────────────▼─────────────┐
│  Flatpak service (typed commands + parsers)      │
│   - list_apps(), install(), progress parsing      │
└───────────────▲───────────────────┬─────────────┘
           spawn│           stderr  │
┌───────────────┴───────────────────▼─────────────┐
│  tokio::process (flatpak CLI subprocesses)        │
└──────────────────────────────────────────────────┘
```

- `flatpak_service` is the only module that spawns subprocesses.
- `app` is the only module that mutates state.
- `ui` is pure functions from `&App` to widgets.

## Domain types

Key types in `flatpak_service/types.rs`:

```rust
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

pub struct Remote {
    pub name: String,
    pub title: String,
    pub url: String,
    pub installation: Installation,
    pub disabled: bool,
    pub priority: i32,
}

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

pub struct Permission {
    pub table: String,
    pub entries: Vec<String>,
}

pub enum Installation { System, User }
pub enum Kind { App, Runtime }
```

## FlatpakService API

All methods are async and return `Result<T, FlatpakError>`.

Queries:
- `list_installed(inst: Option<Installation>) -> Vec<AppRef>`
- `info(ref_: &str) -> AppDetail`
- `list_remotes(inst: Option<Installation>) -> Vec<Remote>`
- `list_history() -> Vec<HistoryEntry>`
- `search(query: &str) -> Vec<SearchHit>`
- `permissions(ref_: &str) -> Vec<Permission>`

Mutations return a `Job` that the caller starts and that emits `JobEvent`:
- `install(remote: &str, ref_: &str, inst: Installation) -> Job`
- `update(ref_: Option<&str>, inst: Installation) -> Job` (`None` means update all)
- `uninstall(ref_: &str, inst: Installation, delete_data: bool) -> Job`

## Parsing strategy

- `flatpak list --columns=...` and `flatpak remotes --columns=...` produce tab-separated machine-readable output. Field names are stable. Parsers read these directly.
- `flatpak info <ref>` is human-readable. The parser is line-oriented, matching `Key: value` pairs. The keys (Ref, Version, Runtime, etc.) are stable across flatpak 1.x.
- Progress parsing scans stderr lines from mutations for patterns like `[####################] 100% Downloading: 42.3 MB`. Emitted as `JobEvent::Progress { pct, line }`. If no percentage is found, `pct` is `None` and the raw line is shown.

## UI structure

Top-level layout:
- Top: title + tab bar (`Apps | Runtimes | Remotes | History | Install`).
- Middle: split list/detail panes. Left ~40%, right ~60%.
- Bottom: status bar with counts, active jobs summary, and key hints.

Tabs:
1. **Apps** — installed apps only.
2. **Runtimes** — installed runtimes; detail shows dependent apps.
3. **Remotes** — remote list; detail shows app count and enable/disable actions.
4. **History** — `flatpak history` in a table.
5. **Install** — search input + results list.

Focus model: focus cycles between tabs, list, detail, and (on Install tab) search. `Tab`/`Shift+Tab` or numeric keys (`1`–`5`) switch tabs. `Esc` returns focus to the tab bar. Focused pane has a bright border; unfocused panes are dim.

Responsive behavior:
- < 100 cols: collapse to single pane; `Enter` opens detail as a full-screen overlay.
- < 60 cols: show "terminal too narrow" message.

Modals:
- Help (`?`) — keybindings.
- Jobs (`J`) — full job list + scrollable output logs.
- Confirm — destructive actions.
- Permissions (`p`) — view app permissions and reset them with `flatpak permission-reset <id>`.

Status bar segments:
- Left: counts from current tab.
- Center: active jobs summary (e.g. "⚙ 2 jobs: updating Zen (45%)…").
- Right: hints ("? help  q quit").

## Keybindings (global)

- `q` — quit
- `?` — help
- `Tab` / `Shift+Tab` — cycle tabs
- `1`–`5` — jump to tab
- `j` / `k` or `↓` / `↑` — move selection
- `Enter` — open detail (single-pane mode) or activate focused control
- `Esc` — back / return focus to tab bar
- `r` — refresh current tab
- `J` — open Jobs modal

Tab-specific:
- Apps/Runtimes: `u` update selected, `Shift+U` update all, `d` uninstall, `p` permissions
- Remotes: `e` enable/disable
- Install: type to search, `Enter` to install selected result

Note: capital letters mean `Shift+<letter>`, e.g. `J` is `Shift+j` and `U` is `Shift+u`.

## Application state & event flow

`App` struct owns:
- `mode`, `tab`, `focus`
- Per-tab state structs (`AppsTab`, `RuntimesTab`, etc.) with items, cursor, detail, filter, loading flag
- `JobManager` for active/completed jobs
- `toast` for transient messages
- `job_rx` and `refresh_rx` channels

Query flow:
1. User triggers refresh (enter tab or press `r`).
2. Loading flag set; UI shows "Loading…".
3. `tokio::spawn` runs the service query and sends `RefreshMsg` to `refresh_tx`.
4. Event loop applies `RefreshMsg`, swapping in new data and clearing loading.

Detail flow:
- Cursor change sets `detail_loading` and spawns `FlatpakService::info(ref)`. If the cursor moves again before the result arrives, the stale result is discarded by comparing `ref`.

Mutation flow:
1. User confirms → `JobManager::start(...)` spawns subprocess.
2. Job emits `JobEvent::Started`, then `JobEvent::Progress`, then `JobEvent::Finished` or `JobEvent::Failed`.
3. Event loop applies events to `JobManager` for the status bar and Jobs modal.
4. On `Finished` for a mutating job, event loop auto-triggers a refresh of the affected tab.

## Job system

A `JobManager` owns:
- A `JoinSet<JobOutcome>` of running jobs.
- An `mpsc::Sender<JobEvent>` that each job clones.
- A `Vec<JobHandle>` for rendering the status bar and Jobs modal. Completed jobs are kept in a capped ring buffer (e.g. last 50).

Each job is an async function that reads its subprocess stdout and stderr (combined) line-by-line, parses progress, and forwards events. It never blocks the event loop.

## Project layout

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
    ├── parse_tests.rs
    ├── service_smoke.rs
    └── job_progress_tests.rs
```

## Crates

- `ratatui` + `crossterm` — TUI rendering and events
- `tokio` — async runtime, process spawning, channels
- `serde` + `serde_json` — config only
- `clap` — CLI flags
- `directories` — XDG dirs
- `tracing` + `tracing-subscriber` — structured logging to file
- `anyhow` / `thiserror` — error handling
- `indexmap` — stable ordered maps
- `chrono` — date/time parsing in info/history

## CLI flags

- `--user` / `--system` / `--installation=NAME` — default installation
- `--no-system` — user installation only
- `-v` / `--verbose` — mirror logs to stderr
- `--version`

## Error handling

- Service layer: `FlatpakError` with `Cli { code, stderr }`, `Parse { line, msg }`, `NotFound`, `Io(...)`.
- Query errors: become a `Toast` with the error text; tab data remains as-is; user retries with `r`.
- Job errors: emitted as `JobEvent::Failed { msg }`; job shown as failed in Jobs modal with its output log.
- Terminal restoration: a `Drop` guard disables raw mode and leaves alternate screen on exit or panic.
- Panics: panic hook restores terminal before printing. Panics are reserved for programmer errors; all user-facing failures use `Result`.

## Logging

`tracing` writes to a rolling file in `$XDG_CACHE_HOME/flatpakmgr/flatpakmgr.log` at debug level. `--verbose` mirrors to stderr. Logs include every subprocess command, parse failures (with offending line), and job state transitions.

## Testing strategy

1. **Parser tests** — primary coverage. Fixtures are captured real `flatpak` outputs in `tests/parse_fixtures/`.
2. **Progress parsing tests** — sample stderr lines → expected `JobEvent::Progress`.
3. **State reducer tests** — `JobManager::apply` and `App::apply_refresh` are pure functions over state.
4. **UI snapshot tests** — `ratatui::backend::TestBackend` asserting rendered text for known `App` states.
5. **Service smoke tests** — `#[ignore]` tests that call live `flatpak`; run with `cargo test -- --ignored`.
6. No full headless TUI integration tests for v1.

## Implementation phases

### Phase 1 — Foundation & read-only flatpak service
- Cargo project, CLI flags, tracing, terminal guard.
- Domain types, `FlatpakError`, pure parsers, parser tests with fixtures.
- `FlatpakService`: `list_installed`, `info`, `list_remotes`, `list_history`, `permissions`, `search`.
- Hidden debug flag `--dump apps` to sanity-check parsing.

### Phase 2 — Minimal TUI: Apps list + detail
- ratatui setup, event loop (frame + input + refresh channels).
- `AppsTab`, list/detail split, selection cursor, refresh, help modal.
- Status bar with app count.

### Phase 3 — Mutations & background jobs
- `Job`, `JobEvent`, `JobManager`, `job_rx` channel.
- `FlatpakService::install`, `update`, `uninstall`.
- Progress parsing, Jobs modal (`J`), confirm modal.
- `u`/`U`/`d` keybindings; auto-refresh on mutation completion.
- Toasts on failure.

### Phase 4 — Runtimes, Remotes, History tabs
- `RuntimesTab`, `RemotesTab`, `HistoryTab` state + render.
- Runtimes: dependent-apps cross-reference.
- Remotes: enable/disable job.
- Tab cycling `1`–`4`/`Tab`.

### Phase 5 — Install tab & search
- `InstallTab` with debounced search input.
- `FlatpakService::search`, result list, `Enter` to install.

### Phase 6 — Polish & resilience
- Permissions view/reset modal.
- Responsive layout (single-pane fallback, too-narrow guard).
- Config file load/save.
- Error toast auto-dismiss, panic hook, README, keybindings reference.

## Open questions / future work

- Whether to add a `flatpak` application manifest for self-distribution. Out of scope for v1.
- Whether to support per-user themes beyond the default palette. Deferred.
