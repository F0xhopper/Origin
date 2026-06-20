//! `origin` — a terminal linguistic time machine. Run `origin <word>` to trace
//! a word backward through history along a horizontal, vim-navigable tree.

use std::process::ExitCode;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{self, Event, KeyEventKind};

use etymology_tui::app::App;
use etymology_tui::cli::Cli;
use etymology_tui::input::map_key;
use etymology_tui::model::Dataset;
use etymology_tui::tui::{self, Tui};
use etymology_tui::ui;

/// How long to wait for input before redrawing (keeps autoplay live).
const POLL: Duration = Duration::from_millis(200);

fn main() -> ExitCode {
    match try_main() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("origin: {err:#}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<ExitCode> {
    let cli = Cli::parse();
    init_logging(&cli)?;

    let dataset = Dataset::load(cli.data.as_deref()).context("loading dataset")?;

    // No word given: print usage-style guidance and exit.
    let Some(query) = cli.word.as_deref() else {
        eprintln!("usage: origin <word>");
        eprintln!("       e.g. `origin salary`, `origin disaster`, `origin muscle`");
        return Ok(ExitCode::from(2));
    };

    // Resolve the word, or suggest near matches and exit.
    let Some(idx) = dataset.resolve(query) else {
        report_not_found(&dataset, query);
        return Ok(ExitCode::FAILURE);
    };

    let word = dataset.get(idx).expect("resolved index is valid").clone();
    let mut app = App::new(word);

    let mouse = !cli.no_mouse;
    tui::install_panic_hook(mouse);
    let mut tui = Tui::enter(mouse).context("entering terminal")?;
    let result = run(&mut tui, &mut app);
    // `tui` restores the terminal on drop here, before any error is printed.
    drop(tui);
    result.map(|()| ExitCode::SUCCESS)
}

/// The main event/render loop.
fn run(tui: &mut Tui, app: &mut App) -> Result<()> {
    while app.running() {
        tui.terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(POLL)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if let Some(action) = map_key(app.overlay, key, &mut app.pending) {
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

/// Tell the user the word wasn't found and offer close matches.
fn report_not_found(dataset: &Dataset, query: &str) {
    eprintln!("origin: no word matching \"{query}\".");
    let suggestions: Vec<&str> = dataset
        .search(query)
        .into_iter()
        .filter_map(|i| dataset.get(i).map(|w| w.headword.as_str()))
        .take(6)
        .collect();
    if !suggestions.is_empty() {
        eprintln!("did you mean: {}?", suggestions.join(", "));
    } else {
        eprintln!(
            "try `origin disaster`, `origin muscle`, or `origin salary` — the dataset covers {} words.",
            dataset.len()
        );
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
