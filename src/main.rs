mod app;
mod keybindings;
mod operations;
mod ui;

use std::io;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, AppResult};
use keybindings::handle_key_event;
use ui::draw_ui;

/// rnm - A modern TUI tool for batch renaming files
#[derive(Parser, Debug)]
#[command(name = "rnm")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory path or glob pattern
    #[arg(default_value = ".")]
    path: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Determine if input is a glob pattern or directory
    let (directory, pattern) = parse_input(&args.path);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(directory, pattern)?;

    // Main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }

    Ok(())
}

fn parse_input(input: &str) -> (PathBuf, Option<String>) {
    // Check if input contains glob characters
    if input.contains('*') || input.contains('?') || input.contains('[') {
        // It's a glob pattern
        let path = PathBuf::from(input);
        if let Some(parent) = path.parent() {
            let dir = if parent.as_os_str().is_empty() {
                PathBuf::from(".")
            } else {
                parent.to_path_buf()
            };
            let pattern = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string());
            (dir, pattern)
        } else {
            (PathBuf::from("."), Some(input.to_string()))
        }
    } else {
        // It's a directory path
        (PathBuf::from(input), None)
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| draw_ui(frame, app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match handle_key_event(app, key) {
                    AppResult::Continue => {}
                    AppResult::Quit => break,
                }
            }
        }
    }

    Ok(())
}

