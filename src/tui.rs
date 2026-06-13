//! Terminal setup/teardown with RAII and panic safety.
//!
//! The [`Tui`] guard enables raw mode, the alternate screen and (optionally)
//! mouse capture on construction and restores everything on `Drop`, so even a
//! panic cannot leave the user's terminal in a broken state.

use std::io::{self, Stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

/// The concrete backend used throughout the app.
pub type Backend = CrosstermBackend<Stdout>;

/// RAII terminal guard.
pub struct Tui {
    /// The ratatui terminal handle.
    pub terminal: Terminal<Backend>,
    mouse: bool,
}

impl Tui {
    /// Enter raw mode + alternate screen (and mouse capture if requested).
    pub fn enter(mouse: bool) -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        if mouse {
            execute!(stdout, EnableMouseCapture)?;
        }
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self { terminal, mouse })
    }

    /// Restore the terminal to its normal state. Idempotent and best-effort.
    pub fn restore(mouse: bool) -> io::Result<()> {
        let mut stdout = io::stdout();
        if mouse {
            let _ = execute!(stdout, DisableMouseCapture);
        }
        execute!(stdout, LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = Self::restore(self.mouse);
    }
}

/// Install a panic hook that restores the terminal before printing the panic,
/// so a crash never bricks the user's shell.
pub fn install_panic_hook(mouse: bool) {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = Tui::restore(mouse);
        default(info);
    }));
}
