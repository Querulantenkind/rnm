use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Result;
use glob::glob;

use crate::operations::RenamePreview;

/// Result of handling a key event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppResult {
    Continue,
    Quit,
}

/// Which panel is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Files,
    SearchField,
    ReplaceField,
}

/// Dialog state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogState {
    None,
    Confirm,
    Help,
    Success,
    Error,
}

/// Represents a file entry
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

/// Main application state
pub struct App {
    /// Current working directory
    pub directory: PathBuf,

    /// List of files in the directory
    pub files: Vec<FileEntry>,

    /// Currently selected file index
    pub selected_index: usize,

    /// Set of selected file indices for batch operations
    pub selected_files: HashSet<usize>,

    /// Current focused panel
    pub focused_panel: FocusedPanel,

    /// Search input field content
    pub search_input: String,

    /// Replace input field content
    pub replace_input: String,

    /// Cursor position in search field
    pub search_cursor: usize,

    /// Cursor position in replace field
    pub replace_cursor: usize,

    /// Preview of rename operations
    pub previews: Vec<RenamePreview>,

    /// Current dialog state
    pub dialog_state: DialogState,

    /// Error message to display
    pub error_message: Option<String>,

    /// Success message to display
    pub success_message: Option<String>,

    /// Number of files renamed in last operation
    pub last_rename_count: usize,
}

impl App {
    pub fn new(directory: PathBuf, pattern: Option<String>) -> Result<Self> {
        let files = load_files(&directory, pattern.as_deref())?;

        Ok(Self {
            directory,
            files,
            selected_index: 0,
            selected_files: HashSet::new(),
            focused_panel: FocusedPanel::Files,
            search_input: String::new(),
            replace_input: String::new(),
            search_cursor: 0,
            replace_cursor: 0,
            previews: Vec::new(),
            dialog_state: DialogState::None,
            error_message: None,
            success_message: None,
            last_rename_count: 0,
        })
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.selected_index < self.files.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Toggle selection of current file
    pub fn toggle_selection(&mut self) {
        if self.files.is_empty() {
            return;
        }

        if self.selected_files.contains(&self.selected_index) {
            self.selected_files.remove(&self.selected_index);
        } else {
            self.selected_files.insert(self.selected_index);
        }
    }

    /// Select all files
    pub fn select_all(&mut self) {
        if self.selected_files.len() == self.files.len() {
            // If all selected, deselect all
            self.selected_files.clear();
        } else {
            // Select all
            self.selected_files = (0..self.files.len()).collect();
        }
    }

    /// Switch to next panel
    pub fn next_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Files => FocusedPanel::SearchField,
            FocusedPanel::SearchField => FocusedPanel::ReplaceField,
            FocusedPanel::ReplaceField => FocusedPanel::Files,
        };
    }

    /// Switch to previous panel
    pub fn previous_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Files => FocusedPanel::ReplaceField,
            FocusedPanel::SearchField => FocusedPanel::Files,
            FocusedPanel::ReplaceField => FocusedPanel::SearchField,
        };
    }

    /// Insert character at cursor position in current input field
    pub fn insert_char(&mut self, c: char) {
        match self.focused_panel {
            FocusedPanel::SearchField => {
                self.search_input.insert(self.search_cursor, c);
                self.search_cursor += 1;
            }
            FocusedPanel::ReplaceField => {
                self.replace_input.insert(self.replace_cursor, c);
                self.replace_cursor += 1;
            }
            FocusedPanel::Files => {}
        }
        self.update_preview();
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        match self.focused_panel {
            FocusedPanel::SearchField => {
                if self.search_cursor > 0 {
                    self.search_cursor -= 1;
                    self.search_input.remove(self.search_cursor);
                }
            }
            FocusedPanel::ReplaceField => {
                if self.replace_cursor > 0 {
                    self.replace_cursor -= 1;
                    self.replace_input.remove(self.replace_cursor);
                }
            }
            FocusedPanel::Files => {}
        }
        self.update_preview();
    }

    /// Move cursor left in input field
    pub fn cursor_left(&mut self) {
        match self.focused_panel {
            FocusedPanel::SearchField => {
                if self.search_cursor > 0 {
                    self.search_cursor -= 1;
                }
            }
            FocusedPanel::ReplaceField => {
                if self.replace_cursor > 0 {
                    self.replace_cursor -= 1;
                }
            }
            FocusedPanel::Files => {}
        }
    }

    /// Move cursor right in input field
    pub fn cursor_right(&mut self) {
        match self.focused_panel {
            FocusedPanel::SearchField => {
                if self.search_cursor < self.search_input.len() {
                    self.search_cursor += 1;
                }
            }
            FocusedPanel::ReplaceField => {
                if self.replace_cursor < self.replace_input.len() {
                    self.replace_cursor += 1;
                }
            }
            FocusedPanel::Files => {}
        }
    }

    /// Update preview based on current search/replace values
    pub fn update_preview(&mut self) {
        self.previews = crate::operations::generate_previews(
            &self.files,
            &self.selected_files,
            &self.search_input,
            &self.replace_input,
        );
    }

    /// Execute the rename operations
    pub fn execute_rename(&mut self) -> Result<usize> {
        let result = crate::operations::execute_renames(&self.previews, &self.directory);
        
        match &result {
            Ok(count) => {
                self.last_rename_count = *count;
                self.success_message = Some(format!("{} Dateien erfolgreich umbenannt", count));
                self.dialog_state = DialogState::Success;
                
                // Reload files after rename
                if let Ok(files) = load_files(&self.directory, None) {
                    self.files = files;
                    self.selected_files.clear();
                    self.selected_index = 0;
                    self.search_input.clear();
                    self.replace_input.clear();
                    self.search_cursor = 0;
                    self.replace_cursor = 0;
                    self.previews.clear();
                }
            }
            Err(e) => {
                self.error_message = Some(e.to_string());
                self.dialog_state = DialogState::Error;
            }
        }
        
        result
    }

    /// Show confirmation dialog
    pub fn show_confirm_dialog(&mut self) {
        if !self.previews.is_empty() && self.previews.iter().any(|p| p.will_change) {
            self.dialog_state = DialogState::Confirm;
        }
    }

    /// Show help dialog
    pub fn show_help(&mut self) {
        self.dialog_state = DialogState::Help;
    }

    /// Close any open dialog
    pub fn close_dialog(&mut self) {
        self.dialog_state = DialogState::None;
        self.error_message = None;
        self.success_message = None;
    }

    /// Get files that will be affected by the operation
    pub fn get_affected_files(&self) -> Vec<&FileEntry> {
        self.previews
            .iter()
            .filter(|p| p.will_change)
            .filter_map(|p| self.files.iter().find(|f| f.name == p.original_name))
            .collect()
    }

    /// Check if we have any changes to apply
    pub fn has_changes(&self) -> bool {
        self.previews.iter().any(|p| p.will_change)
    }
}

/// Load files from directory with optional glob pattern
fn load_files(directory: &PathBuf, pattern: Option<&str>) -> Result<Vec<FileEntry>> {
    let mut files = Vec::new();

    if let Some(pattern) = pattern {
        // Use glob pattern
        let full_pattern = directory.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        for entry in glob(&pattern_str)? {
            if let Ok(path) = entry {
                if path.is_file() {
                    if let Some(name) = path.file_name() {
                        files.push(FileEntry {
                            path: path.clone(),
                            name: name.to_string_lossy().to_string(),
                            is_dir: false,
                        });
                    }
                }
            }
        }
    } else {
        // List all files in directory
        if directory.is_dir() {
            for entry in std::fs::read_dir(directory)? {
                let entry = entry?;
                let path = entry.path();
                let is_dir = path.is_dir();

                // Skip hidden files
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy().to_string();
                    if !name_str.starts_with('.') {
                        files.push(FileEntry {
                            path,
                            name: name_str,
                            is_dir,
                        });
                    }
                }
            }
        }
    }

    // Sort files: directories first, then alphabetically
    files.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });

    Ok(files)
}

