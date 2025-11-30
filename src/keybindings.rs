use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, AppResult, DialogState, FocusedPanel};

/// Handle a key event and update app state accordingly
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> AppResult {
    // Handle dialog states first
    match app.dialog_state {
        DialogState::Confirm => return handle_confirm_dialog(app, key),
        DialogState::Help => return handle_help_dialog(app, key),
        DialogState::Success | DialogState::Error => return handle_message_dialog(app, key),
        DialogState::None => {}
    }

    // Handle Ctrl+C globally
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return AppResult::Quit;
    }

    match app.focused_panel {
        FocusedPanel::Files => handle_files_panel(app, key),
        FocusedPanel::SearchField | FocusedPanel::ReplaceField => handle_input_field(app, key),
    }
}

/// Handle keys in the files panel
fn handle_files_panel(app: &mut App, key: KeyEvent) -> AppResult {
    match key.code {
        // Quit
        KeyCode::Char('q') => AppResult::Quit,

        // Navigation
        KeyCode::Char('j') | KeyCode::Down => {
            app.select_next();
            AppResult::Continue
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.select_previous();
            AppResult::Continue
        }

        // Selection
        KeyCode::Char(' ') => {
            app.toggle_selection();
            AppResult::Continue
        }
        KeyCode::Char('a') => {
            app.select_all();
            AppResult::Continue
        }

        // Mode cycling
        KeyCode::Char('m') => {
            app.cycle_mode();
            AppResult::Continue
        }

        // Sort cycling
        KeyCode::Char('s') => {
            app.cycle_sort();
            AppResult::Continue
        }

        // Panel navigation
        KeyCode::Tab => {
            app.next_panel();
            AppResult::Continue
        }
        KeyCode::BackTab => {
            app.previous_panel();
            AppResult::Continue
        }

        // Execute rename
        KeyCode::Enter => {
            app.show_confirm_dialog();
            AppResult::Continue
        }

        // Help
        KeyCode::Char('?') => {
            app.show_help();
            AppResult::Continue
        }

        _ => AppResult::Continue,
    }
}

/// Handle keys in input fields (search/replace)
fn handle_input_field(app: &mut App, key: KeyEvent) -> AppResult {
    match key.code {
        // Escape to go back to files panel
        KeyCode::Esc => {
            app.focused_panel = FocusedPanel::Files;
            AppResult::Continue
        }

        // Tab to switch panels
        KeyCode::Tab => {
            app.next_panel();
            AppResult::Continue
        }
        KeyCode::BackTab => {
            app.previous_panel();
            AppResult::Continue
        }

        // Text input
        KeyCode::Char(c) => {
            app.insert_char(c);
            AppResult::Continue
        }

        // Backspace
        KeyCode::Backspace => {
            app.delete_char();
            AppResult::Continue
        }

        // Cursor movement
        KeyCode::Left => {
            app.cursor_left();
            AppResult::Continue
        }
        KeyCode::Right => {
            app.cursor_right();
            AppResult::Continue
        }

        // Execute rename from input field
        KeyCode::Enter => {
            app.show_confirm_dialog();
            AppResult::Continue
        }

        // Help
        KeyCode::F(1) => {
            app.show_help();
            AppResult::Continue
        }

        _ => AppResult::Continue,
    }
}

/// Handle keys in confirmation dialog
fn handle_confirm_dialog(app: &mut App, key: KeyEvent) -> AppResult {
    match key.code {
        // Confirm with Enter or 'y'
        KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
            let _ = app.execute_rename();
            AppResult::Continue
        }

        // Cancel with Escape, 'n', or 'q'
        KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('q') => {
            app.close_dialog();
            AppResult::Continue
        }

        _ => AppResult::Continue,
    }
}

/// Handle keys in help dialog
fn handle_help_dialog(app: &mut App, key: KeyEvent) -> AppResult {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Enter => {
            app.close_dialog();
            AppResult::Continue
        }
        _ => AppResult::Continue,
    }
}

/// Handle keys in message dialogs (success/error)
fn handle_message_dialog(app: &mut App, key: KeyEvent) -> AppResult {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
            app.close_dialog();
            AppResult::Continue
        }
        _ => AppResult::Continue,
    }
}
