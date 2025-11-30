mod app;
mod config;
mod keybindings;
mod operations;
mod ui;

use std::collections::HashSet;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, AppResult, DatePosition, PrefixAction, RenameMode};
use config::{parse_date_position, parse_mode, Config, Preset};
use keybindings::handle_key_event;
use operations::{execute_renames, generate_previews, print_previews};
use ui::draw_ui;

/// rnm - A modern TUI tool for batch renaming files
#[derive(Parser, Debug)]
#[command(name = "rnm")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directory path or glob pattern
    #[arg(default_value = ".")]
    path: String,

    /// Preview changes without actually renaming (dry run)
    #[arg(long, short = 'n')]
    dry_run: bool,

    /// Search pattern for find/replace or regex mode
    #[arg(short, long)]
    search: Option<String>,

    /// Replace pattern for find/replace or regex mode
    #[arg(short, long)]
    replace: Option<String>,

    /// Rename mode: search, regex, numbering, prefix, suffix, upper, lower, title
    #[arg(long, short = 'm')]
    mode: Option<String>,

    /// Pattern for numbering mode (e.g., "photo_###")
    #[arg(long)]
    pattern: Option<String>,

    /// Starting number for numbering mode
    #[arg(long, default_value = "1")]
    start: usize,

    /// Add prefix to filenames
    #[arg(long)]
    prefix: Option<String>,

    /// Add suffix to filenames (before extension)
    #[arg(long)]
    suffix: Option<String>,

    /// Remove prefix from filenames
    #[arg(long)]
    remove_prefix: Option<String>,

    /// Remove suffix from filenames (before extension)
    #[arg(long)]
    remove_suffix: Option<String>,

    /// Use date insertion mode (inserts file modification date)
    #[arg(long)]
    date: bool,

    /// Position for date insertion: prefix, suffix, or replace
    #[arg(long, default_value = "prefix")]
    date_position: String,

    /// Load a saved preset by name
    #[arg(long, short = 'p')]
    preset: Option<String>,

    /// Skip confirmation prompt (use with caution)
    #[arg(long, short = 'y')]
    yes: bool,

    /// Save current settings as a preset
    #[arg(long)]
    save_preset: Option<String>,

    /// List available presets
    #[arg(long)]
    list_presets: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle list-presets command
    if args.list_presets {
        return list_presets();
    }

    // Handle save-preset command
    if let Some(preset_name) = &args.save_preset {
        return save_preset(&args, preset_name);
    }

    // Determine if input is a glob pattern or directory
    let (directory, pattern) = parse_input(&args.path);

    // Check if we should run in non-interactive mode
    let non_interactive = args.search.is_some() 
        || args.mode.is_some() 
        || args.preset.is_some()
        || args.pattern.is_some()
        || args.prefix.is_some()
        || args.suffix.is_some()
        || args.remove_prefix.is_some()
        || args.remove_suffix.is_some()
        || args.date
        || args.dry_run;

    if non_interactive {
        run_non_interactive(&args, directory, pattern)
    } else {
        run_interactive(directory, pattern)
    }
}

/// List available presets
fn list_presets() -> Result<()> {
    let config = Config::load()?;
    
    if config.presets.is_empty() {
        println!("Keine Presets gespeichert.");
        println!("\nErstelle ein Preset mit:");
        println!("  rnm --search 'alt' --replace 'neu' --save-preset mein-preset");
        return Ok(());
    }

    println!("Verfuegbare Presets:\n");
    
    for (name, preset) in &config.presets {
        println!("  {} ", name);
        println!("    Modus: {}", preset.mode.display_name());
        if preset.mode.uses_search_replace() {
            println!("    Suche: '{}'", preset.search);
            println!("    Ersetze: '{}'", preset.replace);
        }
        println!();
    }

    Ok(())
}

/// Save current settings as a preset
fn save_preset(args: &Args, preset_name: &str) -> Result<()> {
    let mut config = Config::load()?;

    let mode = if let Some(mode_str) = &args.mode {
        parse_mode(mode_str).ok_or_else(|| anyhow!("Unbekannter Modus: {}", mode_str))?
    } else {
        RenameMode::SearchReplace
    };

    let preset = Preset::new(
        preset_name.to_string(),
        mode,
        args.search.clone().unwrap_or_default(),
        args.replace.clone().unwrap_or_default(),
    );

    config.add_preset(preset);
    config.save()?;

    println!("Preset '{}' gespeichert.", preset_name);
    Ok(())
}

/// Run in non-interactive mode (CLI)
fn run_non_interactive(args: &Args, directory: PathBuf, pattern: Option<String>) -> Result<()> {
    let config = Config::load()?;

    // Determine mode, search, replace, prefix_action, and date_position from args
    let (mode, search, replace, prefix_action, number_start, date_position) = determine_mode_from_args(args, &config)?;

    // Validate inputs based on mode
    validate_mode_inputs(mode, &search)?;

    // Load files
    let files = app::load_files(&directory, pattern.as_deref(), config.default_sort)?;
    
    if files.is_empty() {
        println!("Keine Dateien gefunden.");
        return Ok(());
    }

    println!("Verzeichnis: {}", directory.display());
    println!("Modus: {}", mode.display_name());
    print_mode_details(mode, &search, &replace, prefix_action, date_position);
    println!("Dateien: {}", files.len());

    // Generate previews
    let selected: HashSet<usize> = HashSet::new();
    let previews = generate_previews(&files, &selected, &search, &replace, mode, prefix_action, number_start, 1, date_position)?;

    // Print preview
    print_previews(&previews);

    let changes: Vec<_> = previews.iter().filter(|p| p.will_change).collect();
    
    if changes.is_empty() {
        return Ok(());
    }

    // Dry run - just show preview
    if args.dry_run {
        println!("(Dry-Run: Keine Aenderungen vorgenommen)");
        return Ok(());
    }

    // Confirmation
    if !args.yes {
        print!("Fortfahren? [y/N] ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Abgebrochen.");
            return Ok(());
        }
    }

    // Execute renames
    let count = execute_renames(&previews, &directory)?;
    println!("{} Datei(en) erfolgreich umbenannt.", count);

    Ok(())
}

/// Determine mode and settings from CLI arguments
fn determine_mode_from_args(args: &Args, config: &Config) -> Result<(RenameMode, String, String, PrefixAction, usize, DatePosition)> {
    // Parse date position
    let date_position = parse_date_position(&args.date_position)
        .ok_or_else(|| anyhow!("Unbekannte Datums-Position: {} (erlaubt: prefix, suffix, replace)", args.date_position))?;

    // Check for preset first
    if let Some(preset_name) = &args.preset {
        let preset = config.get_preset(preset_name)
            .ok_or_else(|| anyhow!("Preset nicht gefunden: {}", preset_name))?;
        return Ok((preset.mode, preset.search.clone(), preset.replace.clone(), PrefixAction::Add, 1, date_position));
    }

    // Check for shortcut arguments
    if args.date {
        return Ok((RenameMode::DateInsert, String::new(), String::new(), PrefixAction::Add, 1, date_position));
    }
    if let Some(prefix) = &args.prefix {
        return Ok((RenameMode::Prefix, prefix.clone(), String::new(), PrefixAction::Add, 1, date_position));
    }
    if let Some(suffix) = &args.suffix {
        return Ok((RenameMode::Suffix, suffix.clone(), String::new(), PrefixAction::Add, 1, date_position));
    }
    if let Some(prefix) = &args.remove_prefix {
        return Ok((RenameMode::Prefix, prefix.clone(), String::new(), PrefixAction::Remove, 1, date_position));
    }
    if let Some(suffix) = &args.remove_suffix {
        return Ok((RenameMode::Suffix, suffix.clone(), String::new(), PrefixAction::Remove, 1, date_position));
    }
    if let Some(pattern) = &args.pattern {
        return Ok((RenameMode::Numbering, pattern.clone(), String::new(), PrefixAction::Add, args.start, date_position));
    }

    // Use explicit mode
    let mode = if let Some(mode_str) = &args.mode {
        parse_mode(mode_str).ok_or_else(|| anyhow!("Unbekannter Modus: {}", mode_str))?
    } else {
        RenameMode::SearchReplace
    };

    Ok((
        mode,
        args.search.clone().unwrap_or_default(),
        args.replace.clone().unwrap_or_default(),
        PrefixAction::Add,
        args.start,
        date_position,
    ))
}

/// Validate inputs based on mode
fn validate_mode_inputs(mode: RenameMode, search: &str) -> Result<()> {
    match mode {
        RenameMode::SearchReplace | RenameMode::Regex => {
            if search.is_empty() {
                return Err(anyhow!("Fuer diesen Modus muss --search angegeben werden"));
            }
        }
        RenameMode::Numbering => {
            if search.is_empty() {
                return Err(anyhow!("Fuer Nummerierung muss --pattern angegeben werden"));
            }
        }
        RenameMode::Prefix | RenameMode::Suffix => {
            if search.is_empty() {
                return Err(anyhow!("Fuer Prefix/Suffix muss ein Wert angegeben werden"));
            }
        }
        RenameMode::DateInsert => {
            // No additional validation needed for date mode
        }
        _ => {}
    }
    Ok(())
}

/// Print mode-specific details
fn print_mode_details(mode: RenameMode, search: &str, replace: &str, prefix_action: PrefixAction, date_position: DatePosition) {
    match mode {
        RenameMode::SearchReplace => {
            println!("Suche: '{}' -> Ersetze: '{}'", search, replace);
        }
        RenameMode::Regex => {
            println!("Regex: '{}' -> '{}'", search, replace);
        }
        RenameMode::Numbering => {
            println!("Muster: '{}'", search);
        }
        RenameMode::Prefix | RenameMode::Suffix => {
            let action = if prefix_action == PrefixAction::Add { "Hinzufuegen" } else { "Entfernen" };
            println!("{}: '{}' ({})", mode.display_name(), search, action);
        }
        RenameMode::DateInsert => {
            println!("Position: {} (Format: YYYYMMDD)", date_position.display_name());
        }
        _ => {}
    }
}

/// Run in interactive TUI mode
fn run_interactive(directory: PathBuf, pattern: Option<String>) -> Result<()> {
    // Load config for defaults
    let config = Config::load().unwrap_or_default();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(directory, pattern)?;
    
    // Apply config defaults
    app.rename_mode = config.default_mode;
    app.sort_order = config.default_sort;
    app.apply_sort();

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
