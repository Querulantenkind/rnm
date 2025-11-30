use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::Result;
use glob::glob;
use serde::{Deserialize, Serialize};

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

/// Action for prefix/suffix mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PrefixAction {
    #[default]
    Add,
    Remove,
}

impl PrefixAction {
    pub fn toggle(&self) -> Self {
        match self {
            PrefixAction::Add => PrefixAction::Remove,
            PrefixAction::Remove => PrefixAction::Add,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PrefixAction::Add => "Hinzufuegen",
            PrefixAction::Remove => "Entfernen",
        }
    }
}

/// Rename operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RenameMode {
    #[default]
    SearchReplace,
    Regex,
    Numbering,
    Prefix,
    Suffix,
    Uppercase,
    Lowercase,
    TitleCase,
}

impl RenameMode {
    /// Cycle to the next mode
    pub fn next(&self) -> Self {
        match self {
            RenameMode::SearchReplace => RenameMode::Regex,
            RenameMode::Regex => RenameMode::Numbering,
            RenameMode::Numbering => RenameMode::Prefix,
            RenameMode::Prefix => RenameMode::Suffix,
            RenameMode::Suffix => RenameMode::Uppercase,
            RenameMode::Uppercase => RenameMode::Lowercase,
            RenameMode::Lowercase => RenameMode::TitleCase,
            RenameMode::TitleCase => RenameMode::SearchReplace,
        }
    }

    /// Get display name for the mode
    pub fn display_name(&self) -> &'static str {
        match self {
            RenameMode::SearchReplace => "Suchen/Ersetzen",
            RenameMode::Regex => "Regex",
            RenameMode::Numbering => "Nummerierung",
            RenameMode::Prefix => "Prefix",
            RenameMode::Suffix => "Suffix",
            RenameMode::Uppercase => "GROSSBUCHSTABEN",
            RenameMode::Lowercase => "kleinbuchstaben",
            RenameMode::TitleCase => "Titel Schreibweise",
        }
    }

    /// Check if this mode uses search/replace fields
    pub fn uses_search_replace(&self) -> bool {
        matches!(self, RenameMode::SearchReplace | RenameMode::Regex)
    }

    /// Check if this mode uses input fields at all
    pub fn uses_input(&self) -> bool {
        matches!(
            self,
            RenameMode::SearchReplace
                | RenameMode::Regex
                | RenameMode::Numbering
                | RenameMode::Prefix
                | RenameMode::Suffix
        )
    }
}

/// Sort order for file list
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SortOrder {
    #[default]
    Name,
    NameDesc,
    Size,
    SizeDesc,
    Extension,
    Date,
    DateDesc,
}

impl SortOrder {
    /// Cycle to the next sort order
    pub fn next(&self) -> Self {
        match self {
            SortOrder::Name => SortOrder::NameDesc,
            SortOrder::NameDesc => SortOrder::Size,
            SortOrder::Size => SortOrder::SizeDesc,
            SortOrder::SizeDesc => SortOrder::Extension,
            SortOrder::Extension => SortOrder::Date,
            SortOrder::Date => SortOrder::DateDesc,
            SortOrder::DateDesc => SortOrder::Name,
        }
    }

    /// Get display name for the sort order
    pub fn display_name(&self) -> &'static str {
        match self {
            SortOrder::Name => "Name A-Z",
            SortOrder::NameDesc => "Name Z-A",
            SortOrder::Size => "Groesse +",
            SortOrder::SizeDesc => "Groesse -",
            SortOrder::Extension => "Erweiterung",
            SortOrder::Date => "Datum alt",
            SortOrder::DateDesc => "Datum neu",
        }
    }

    /// Get short indicator for title bar
    pub fn short_indicator(&self) -> &'static str {
        match self {
            SortOrder::Name => "[A-Z]",
            SortOrder::NameDesc => "[Z-A]",
            SortOrder::Size => "[S+]",
            SortOrder::SizeDesc => "[S-]",
            SortOrder::Extension => "[Ext]",
            SortOrder::Date => "[D+]",
            SortOrder::DateDesc => "[D-]",
        }
    }
}

/// Represents a file entry
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<SystemTime>,
    pub extension: String,
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

    /// Search input field content (also used for pattern/prefix/suffix)
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

    /// Current rename mode
    pub rename_mode: RenameMode,

    /// Current sort order
    pub sort_order: SortOrder,

    /// Action for prefix/suffix mode
    pub prefix_action: PrefixAction,

    /// Starting number for numbering mode
    pub number_start: usize,

    /// Step for numbering mode
    pub number_step: usize,

    /// Regex error message (if pattern is invalid)
    pub regex_error: Option<String>,
}

impl App {
    pub fn new(directory: PathBuf, pattern: Option<String>) -> Result<Self> {
        let files = load_files(&directory, pattern.as_deref(), SortOrder::Name)?;

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
            rename_mode: RenameMode::default(),
            sort_order: SortOrder::default(),
            prefix_action: PrefixAction::default(),
            number_start: 1,
            number_step: 1,
            regex_error: None,
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
            FocusedPanel::Files => {
                if self.rename_mode.uses_input() {
                    FocusedPanel::SearchField
                } else {
                    FocusedPanel::Files
                }
            }
            FocusedPanel::SearchField => {
                if self.rename_mode.uses_search_replace() {
                    FocusedPanel::ReplaceField
                } else {
                    FocusedPanel::Files
                }
            }
            FocusedPanel::ReplaceField => FocusedPanel::Files,
        };
    }

    /// Switch to previous panel
    pub fn previous_panel(&mut self) {
        self.focused_panel = match self.focused_panel {
            FocusedPanel::Files => {
                if self.rename_mode.uses_search_replace() {
                    FocusedPanel::ReplaceField
                } else if self.rename_mode.uses_input() {
                    FocusedPanel::SearchField
                } else {
                    FocusedPanel::Files
                }
            }
            FocusedPanel::SearchField => FocusedPanel::Files,
            FocusedPanel::ReplaceField => FocusedPanel::SearchField,
        };
    }

    /// Cycle to next rename mode
    pub fn cycle_mode(&mut self) {
        self.rename_mode = self.rename_mode.next();
        // Reset to files panel if mode doesn't use input
        if !self.rename_mode.uses_input() {
            self.focused_panel = FocusedPanel::Files;
        }
        // Clear regex error when switching modes
        self.regex_error = None;
        // Set default pattern for numbering mode
        if self.rename_mode == RenameMode::Numbering && self.search_input.is_empty() {
            self.search_input = "file_###".to_string();
            self.search_cursor = self.search_input.len();
        }
        self.update_preview();
    }

    /// Toggle prefix/suffix action (add/remove)
    pub fn toggle_prefix_action(&mut self) {
        self.prefix_action = self.prefix_action.toggle();
        self.update_preview();
    }

    /// Cycle to next sort order
    pub fn cycle_sort(&mut self) {
        self.sort_order = self.sort_order.next();
        self.apply_sort();
    }

    /// Apply current sort order to files
    pub fn apply_sort(&mut self) {
        sort_files(&mut self.files, self.sort_order);
        // Reset selection after sort
        self.selected_files.clear();
        self.selected_index = 0;
        self.update_preview();
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
        let result = crate::operations::generate_previews(
            &self.files,
            &self.selected_files,
            &self.search_input,
            &self.replace_input,
            self.rename_mode,
            self.prefix_action,
            self.number_start,
            self.number_step,
        );

        match result {
            Ok(previews) => {
                self.previews = previews;
                self.regex_error = None;
            }
            Err(e) => {
                self.previews = Vec::new();
                self.regex_error = Some(e.to_string());
            }
        }
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
                if let Ok(files) = load_files(&self.directory, None, self.sort_order) {
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
        // Update preview before showing dialog
        self.update_preview();
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

    /// Set rename mode directly
    pub fn set_mode(&mut self, mode: RenameMode) {
        self.rename_mode = mode;
        if !mode.uses_input() {
            self.focused_panel = FocusedPanel::Files;
        }
        self.update_preview();
    }

    /// Set search and replace values
    pub fn set_search_replace(&mut self, search: String, replace: String) {
        self.search_input = search;
        self.replace_input = replace;
        self.search_cursor = self.search_input.len();
        self.replace_cursor = self.replace_input.len();
        self.update_preview();
    }
}

/// Load files from directory with optional glob pattern
pub fn load_files(directory: &PathBuf, pattern: Option<&str>, sort_order: SortOrder) -> Result<Vec<FileEntry>> {
    let mut files = Vec::new();

    if let Some(pattern) = pattern {
        // Use glob pattern
        let full_pattern = directory.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        for entry in glob(&pattern_str)? {
            if let Ok(path) = entry {
                if path.is_file() {
                    if let Some(name) = path.file_name() {
                        let metadata = std::fs::metadata(&path).ok();
                        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                        let modified = metadata.as_ref().and_then(|m| m.modified().ok());
                        let extension = path
                            .extension()
                            .map(|e| e.to_string_lossy().to_string())
                            .unwrap_or_default();

                        files.push(FileEntry {
                            path: path.clone(),
                            name: name.to_string_lossy().to_string(),
                            is_dir: false,
                            size,
                            modified,
                            extension,
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
                        let metadata = std::fs::metadata(&path).ok();
                        let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                        let modified = metadata.as_ref().and_then(|m| m.modified().ok());
                        let extension = path
                            .extension()
                            .map(|e| e.to_string_lossy().to_string())
                            .unwrap_or_default();

                        files.push(FileEntry {
                            path,
                            name: name_str,
                            is_dir,
                            size,
                            modified,
                            extension,
                        });
                    }
                }
            }
        }
    }

    // Apply sorting
    sort_files(&mut files, sort_order);

    Ok(files)
}

/// Sort files according to the given order
fn sort_files(files: &mut Vec<FileEntry>, sort_order: SortOrder) {
    files.sort_by(|a, b| {
        // Directories always come first
        match (a.is_dir, b.is_dir) {
            (true, false) => return std::cmp::Ordering::Less,
            (false, true) => return std::cmp::Ordering::Greater,
            _ => {}
        }

        match sort_order {
            SortOrder::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortOrder::NameDesc => b.name.to_lowercase().cmp(&a.name.to_lowercase()),
            SortOrder::Size => a.size.cmp(&b.size),
            SortOrder::SizeDesc => b.size.cmp(&a.size),
            SortOrder::Extension => a.extension.to_lowercase().cmp(&b.extension.to_lowercase()),
            SortOrder::Date => a.modified.cmp(&b.modified),
            SortOrder::DateDesc => b.modified.cmp(&a.modified),
        }
    });
}
