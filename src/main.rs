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
