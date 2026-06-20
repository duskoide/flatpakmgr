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
use std::time::Duration;

use flatpakmgr::app;
use flatpakmgr::flatpak_service;

mod config;
mod telemetry;

#[derive(Parser)]
#[command(name = "flatpakmgr", version)]
struct Cli {
    #[arg(long, group = "install_target")]
    user: bool,
    #[arg(long, group = "install_target")]
    system: bool,
    #[arg(long, group = "install_target")]
    installation: Option<String>,
    #[arg(long)]
    no_system: bool,
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let result = run(&mut terminal).await;

    restore_terminal()?;
    result
}

fn restore_terminal() -> anyhow::Result<()> {
    terminal::disable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

async fn run<B: Backend>(terminal: &mut Terminal<B>) -> anyhow::Result<()> {
    use app::input::handle_input;
    use app::App;
    use crossterm::event::EventStream;
    use futures::StreamExt;
    use tokio::time::interval;

    let mut app = App::new();
    let mut events = EventStream::new();
    let mut tick = interval(Duration::from_secs_f32(1.0 / 30.0));

    app::start_apps_refresh(&mut app);

    while !app.should_quit {
        let size = terminal.size()?;
        app.last_width = size.width;
        app.last_height = size.height;
        app.clear_expired_toast();

        tokio::select! {
            _ = tick.tick() => {
                terminal.draw(|frame| flatpakmgr::ui::draw(frame, &app))?;
            }
            Some(Ok(event)) = events.next() => {
                handle_input(&mut app, event);
            }
            Some(job_evt) = app.job_rx.recv() => {
                match &job_evt {
                    flatpak_service::job::JobEvent::Finished { .. } => {
                        app.jobs.apply(&job_evt);
                        app::start_apps_refresh(&mut app);
                    }
                    flatpak_service::job::JobEvent::Failed { id, msg } => {
                        app.jobs.apply(&job_evt);
                        app.set_toast(app::Toast::Error(format!("Job {:?} failed: {}", id, msg)));
                        app::start_apps_refresh(&mut app);
                    }
                    _ => app.jobs.apply(&job_evt),
                }
            }
            Some(refresh) = app.refresh_rx.recv() => {
                apply_refresh(&mut app, refresh);
            }
        }
    }
    Ok(())
}

fn apply_refresh(app: &mut app::App, msg: app::RefreshMsg) {
    match msg {
        app::RefreshMsg::Apps(items) => {
            app.apps.items = items;
            app.apps.loading = false;
            app.apps.cursor = 0;
            if let Some(app_ref) = app.apps.selected_ref().cloned() {
                app::start_app_detail_refresh(app, app_ref);
            }
        }
        app::RefreshMsg::AppDetail { app_ref, detail } => {
            if app.apps.selected_ref().map(|a| a.ref_.as_str()) == Some(app_ref.ref_.as_str()) {
                match detail {
                    Ok(d) => app.apps.detail = Some(d),
                    Err(e) => app.set_toast(app::Toast::Error(e.to_string())),
                }
                app.apps.detail_loading = false;
            }
        }
        app::RefreshMsg::Runtimes(items) => {
            app.runtimes.items = items;
            app.runtimes.loading = false;
            app.runtimes.cursor = 0;
        }
        app::RefreshMsg::Remotes(items) => {
            app.remotes.items = items;
            app.remotes.loading = false;
            app.remotes.cursor = 0;
        }
        app::RefreshMsg::History(items) => {
            app.history.items = items;
            app.history.loading = false;
            app.history.cursor = 0;
        }
        app::RefreshMsg::SearchResults { token, results } => {
            if token == app.install.debounce_token {
                match results {
                    Ok(items) => app.install.results = items,
                    Err(e) => app.set_toast(app::Toast::Error(e.to_string())),
                }
                app.install.loading = false;
            }
        }
    }
}
