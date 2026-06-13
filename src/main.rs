//! Reverse Etymology Timeline TUI — a terminal linguistic time machine.

use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{self, Event, KeyEventKind};

use etymology_tui::app::App;
use etymology_tui::cli::Cli;
use etymology_tui::input::{map_key, Action};
use etymology_tui::model::Dataset;
use etymology_tui::tui::{self, Tui};
use etymology_tui::ui;

/// How long to wait for input before redrawing (keeps the clock & autoplay live).
const POLL: Duration = Duration::from_millis(200);

fn main() -> Result<()> {
    let cli = Cli::parse();
    init_logging(&cli)?;

    let dataset = Dataset::load(cli.data.as_deref()).context("loading dataset")?;
    let mut app = App::new(dataset);
    apply_startup(&mut app, &cli);

    let mouse = !cli.no_mouse;
    tui::install_panic_hook(mouse);

    let mut tui = Tui::enter(mouse).context("entering terminal")?;
    let result = run(&mut tui, &mut app);
    // `tui` restores the terminal on drop here, before any error is printed.
    drop(tui);
    result
}

/// The main event/render loop.
fn run(tui: &mut Tui, app: &mut App) -> Result<()> {
    while app.running() {
        tui.terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(POLL)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if let Some(action) = map_key(app.mode, app.overlay, key, &mut app.pending) {
                        app.update(action);
                    }
                }
                Event::Mouse(me) => app.handle_mouse(me),
                Event::Resize(_, _) => {} // redrawn next iteration
                _ => {}
            }
        }

        app.tick();
    }
    Ok(())
}

/// Honour `--word` / `--random` startup options.
fn apply_startup(app: &mut App, cli: &Cli) {
    if let Some(id) = &cli.word {
        if let Some(idx) = app.dataset.index_of_id(id) {
            if let Some(pos) = app.filtered.iter().position(|&i| i == idx) {
                app.list_cursor = pos;
            }
            app.update(Action::Open);
        }
    } else if cli.random {
        app.update(Action::Random);
    }
}

/// Configure file logging if requested. No-op otherwise (never logs to the TUI).
fn init_logging(cli: &Cli) -> Result<()> {
    if let Some(path) = &cli.log {
        let file = std::fs::File::create(path)
            .with_context(|| format!("creating log file {}", path.display()))?;
        tracing_subscriber::fmt()
            .with_writer(move || file.try_clone().expect("clone log file handle"))
            .with_ansi(false)
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .init();
        tracing::info!("logging initialised");
    }
    Ok(())
}
