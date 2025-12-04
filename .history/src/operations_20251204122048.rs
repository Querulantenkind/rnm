use std::collections::HashSet;
use std::path::PathBuf;
use std::time::SystemTime;

use anyhow::{anyhow, Result};
use regex::Regex;

use crate::app::{DatePosition, FileEntry, PrefixAction, RenameMode};
use crate::config::{RenameHistory, RenameHistoryEntry, RenameOperation};

/// Preview of a rename operation
#[derive(Debug, Clone)]
pub struct RenamePreview {
    /// Original filename
    pub original_name: String,
    /// New filename after rename
    pub new_name: String,
    /// Whether this file will actually change
    pub will_change: bool,
    /// Index of the file in the original list
    pub file_index: usize,
}

/// Generate previews for all selected files based on mode and search/replace
pub fn generate_previews(
    files: &[FileEntry],
    selected: &HashSet<usize>,
    search: &str,
    replace: &str,
    mode: RenameMode,
    prefix_action: PrefixAction,
    number_start: usize,
    number_step: usize,
    date_position: DatePosition,
) -> Result<Vec<RenamePreview>> {
    let mut previews = Vec::new();

    // If nothing is selected, preview all files
    let indices: Vec<usize> = if selected.is_empty() {
        (0..files.len()).collect()
    } else {
        let mut v: Vec<usize> = selected.iter().copied().collect();
        v.sort(); // Sort for consistent numbering
        v
    };

    // Pre-compile regex if in regex mode
    let regex = if mode == RenameMode::Regex && !search.is_empty() {
        Some(Regex::new(search).map_err(|e| anyhow!("Ungueltiger Regex: {}", e))?)
    } else {
        None
    };

    // Counter for numbering mode
    let mut counter = number_start;

    for index in indices {
        if let Some(file) = files.get(index) {
            // Skip directories for now
            if file.is_dir {
                continue;
            }

            let new_name = apply_rename_mode(
                &file.name,
                search,
                replace,
                mode,
                prefix_action,
                regex.as_ref(),
                counter,
                date_position,
                file.modified,
            );

            let will_change = new_name != file.name;

            previews.push(RenamePreview {
                original_name: file.name.clone(),
                new_name,
                will_change,
                file_index: index,
            });

            // Increment counter for numbering mode
            if mode == RenameMode::Numbering {
                counter += number_step;
            }
        }
    }

    // Sort by original name for display
    previews.sort_by(|a, b| a.original_name.cmp(&b.original_name));

    Ok(previews)
}

/// Apply the rename mode to a filename
fn apply_rename_mode(
    filename: &str,
    search: &str,
    replace: &str,
    mode: RenameMode,
    prefix_action: PrefixAction,
    regex: Option<&Regex>,
    counter: usize,
    date_position: DatePosition,
    modified: Option<SystemTime>,
) -> String {
    match mode {
        RenameMode::SearchReplace => {
            if search.is_empty() {
                filename.to_string()
            } else {
                filename.replace(search, replace)
            }
        }
        RenameMode::Regex => {
            if let Some(re) = regex {
                re.replace_all(filename, replace).to_string()
            } else {
                filename.to_string()
            }
        }
        RenameMode::Numbering => apply_numbering(filename, search, counter),
        RenameMode::Prefix => apply_prefix(filename, search, prefix_action),
        RenameMode::Suffix => apply_suffix(filename, search, prefix_action),
        RenameMode::DateInsert => apply_date_insert(filename, date_position, modified),
        RenameMode::Uppercase => to_uppercase_preserve_extension(filename),
        RenameMode::Lowercase => to_lowercase_preserve_extension(filename),
        RenameMode::TitleCase => to_titlecase_preserve_extension(filename),
    }
}

/// Apply numbering pattern to filename
/// Pattern uses # for digits: file_### -> file_001, file_002, etc.
fn apply_numbering(filename: &str, pattern: &str, counter: usize) -> String {
    if pattern.is_empty() {
        return filename.to_string();
    }

    // Find the extension of the original file
    let extension = if let Some(dot_pos) = filename.rfind('.') {
        &filename[dot_pos..]
    } else {
        ""
    };

    // Count consecutive # characters to determine padding
    let hash_count = pattern.chars().filter(|&c| c == '#').count();

    if hash_count == 0 {
        // No # in pattern, just use pattern as-is with extension
        return format!("{}{}", pattern, extension);
    }

    // Replace # sequence with padded number
    let mut result = String::new();
    let mut in_hash_sequence = false;
    let mut hash_start = 0;

    for (i, c) in pattern.chars().enumerate() {
        if c == '#' {
            if !in_hash_sequence {
                in_hash_sequence = true;
                hash_start = i;
            }
        } else {
            if in_hash_sequence {
                // End of hash sequence, insert padded number
                let padding = i - hash_start;
                result.push_str(&format!("{:0>width$}", counter, width = padding));
                in_hash_sequence = false;
            }
            result.push(c);
        }
    }

    // Handle trailing hash sequence
    if in_hash_sequence {
        let padding = pattern.len() - hash_start;
        result.push_str(&format!("{:0>width$}", counter, width = padding));
    }

    // Add extension
    format!("{}{}", result, extension)
}

/// Apply prefix to filename
fn apply_prefix(filename: &str, prefix: &str, action: PrefixAction) -> String {
    if prefix.is_empty() {
        return filename.to_string();
    }

    match action {
        PrefixAction::Add => format!("{}{}", prefix, filename),
        PrefixAction::Remove => {
            if filename.starts_with(prefix) {
                filename[prefix.len()..].to_string()
            } else {
                filename.to_string()
            }
        }
    }
}

/// Apply suffix to filename (before extension)
fn apply_suffix(filename: &str, suffix: &str, action: PrefixAction) -> String {
    if suffix.is_empty() {
        return filename.to_string();
    }

    // Split filename and extension
    let (name, ext) = if let Some(dot_pos) = filename.rfind('.') {
        (&filename[..dot_pos], &filename[dot_pos..])
    } else {
        (filename, "")
    };

    match action {
        PrefixAction::Add => format!("{}{}{}", name, suffix, ext),
        PrefixAction::Remove => {
            if name.ends_with(suffix) {
                format!("{}{}", &name[..name.len() - suffix.len()], ext)
            } else {
                filename.to_string()
            }
        }
    }
}

/// Format SystemTime as YYYYMMDD string
fn format_date(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    // Calculate date from SystemTime
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    // Simple date calculation (not accounting for leap seconds, but good enough)
    let days = secs / 86400;

    // Calculate year, month, day
    let mut year = 1970;
    let mut remaining_days = days as i64;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let is_leap = is_leap_year(year);
    let days_in_months: [i64; 12] = [
        31,
        if is_leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 1;
    for days_in_month in days_in_months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days + 1;

    format!("{:04}{:02}{:02}", year, month, day)
}

/// Check if a year is a leap year
fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Apply date insertion to filename
fn apply_date_insert(
    filename: &str,
    position: DatePosition,
    modified: Option<SystemTime>,
) -> String {
    let date_str = match modified {
        Some(time) => format_date(time),
        None => "00000000".to_string(), // Fallback if no date available
    };

    // Split filename and extension
    let (name, ext) = if let Some(dot_pos) = filename.rfind('.') {
        (&filename[..dot_pos], &filename[dot_pos..])
    } else {
        (filename, "")
    };

    match position {
        DatePosition::Prefix => format!("{}_{}{}", date_str, name, ext),
        DatePosition::Suffix => format!("{}_{}{}", name, date_str, ext),
        DatePosition::Replace => format!("{}{}", date_str, ext),
    }
}

/// Convert filename to uppercase, preserving extension case optionally
fn to_uppercase_preserve_extension(filename: &str) -> String {
    if let Some(dot_pos) = filename.rfind('.') {
        let (name, ext) = filename.split_at(dot_pos);
        format!("{}{}", name.to_uppercase(), ext.to_lowercase())
    } else {
        filename.to_uppercase()
    }
}

/// Convert filename to lowercase
fn to_lowercase_preserve_extension(filename: &str) -> String {
    filename.to_lowercase()
}

/// Convert filename to title case
fn to_titlecase_preserve_extension(filename: &str) -> String {
    if let Some(dot_pos) = filename.rfind('.') {
        let (name, ext) = filename.split_at(dot_pos);
        format!("{}{}", to_titlecase(name), ext.to_lowercase())
    } else {
        to_titlecase(filename)
    }
}

/// Convert a string to title case
fn to_titlecase(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = true;

    for c in s.chars() {
        if c.is_whitespace() || c == '_' || c == '-' {
            result.push(c);
            capitalize_next = true;
        } else if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.extend(c.to_lowercase());
        }
    }

    result
}

/// Execute the actual rename operations and record history
pub fn execute_renames(previews: &[RenamePreview], directory: &PathBuf) -> Result<usize> {
    execute_renames_with_history(previews, directory, Some("Umbenennung"))
}

/// Execute the actual rename operations with optional history recording
pub fn execute_renames_with_history(
    previews: &[RenamePreview],
    directory: &PathBuf,
    description: Option<&str>,
) -> Result<usize> {
    let mut renamed_count = 0;
    let mut errors = Vec::new();
    let mut history_entries = Vec::new();

    // First, validate all operations
    for preview in previews.iter().filter(|p| p.will_change) {
        let old_path = directory.join(&preview.original_name);
        let new_path = directory.join(&preview.new_name);

        // Check if source exists
        if !old_path.exists() {
            errors.push(format!(
                "Quelldatei existiert nicht: {}",
                preview.original_name
            ));
            continue;
        }

        // Check if target already exists (and is different from source)
        if new_path.exists() && old_path != new_path {
            // Case-insensitive check for case changes
            if old_path.to_string_lossy().to_lowercase()
                != new_path.to_string_lossy().to_lowercase()
            {
                errors.push(format!("Zieldatei existiert bereits: {}", preview.new_name));
                continue;
            }
        }

        // Check for invalid characters in new name
        if preview.new_name.contains('/') || preview.new_name.contains('\\') {
            errors.push(format!("Ungueltiger Dateiname: {}", preview.new_name));
            continue;
        }

        // Check for empty filename
        if preview.new_name.is_empty() {
            errors.push("Leerer Dateiname ist nicht erlaubt".to_string());
            continue;
        }
    }

    if !errors.is_empty() {
        return Err(anyhow!("Validierungsfehler:\n{}", errors.join("\n")));
    }

    // Execute renames
    for preview in previews.iter().filter(|p| p.will_change) {
        let old_path = directory.join(&preview.original_name);
        let new_path = directory.join(&preview.new_name);

        match std::fs::rename(&old_path, &new_path) {
            Ok(_) => {
                renamed_count += 1;
                // Record for history
                history_entries.push(RenameHistoryEntry {
                    original_name: preview.original_name.clone(),
                    new_name: preview.new_name.clone(),
                });
            }
            Err(e) => {
                return Err(anyhow!(
                    "Fehler beim Umbenennen von '{}' zu '{}': {}",
                    preview.original_name,
                    preview.new_name,
                    e
                ));
            }
        }
    }

    // Save to history if we renamed any files
    if !history_entries.is_empty() && description.is_some() {
        if let Ok(mut history) = RenameHistory::load() {
            let operation = RenameOperation::new(
                directory.clone(),
                history_entries,
                description.unwrap_or("Umbenennung").to_string(),
            );
            history.add_operation(operation);
            let _ = history.save(); // Ignore save errors to not break the main operation
        }
    }

    Ok(renamed_count)
}

/// Undo the last rename operation
pub fn undo_last_rename() -> Result<(usize, PathBuf)> {
    let mut history = RenameHistory::load()?;

    let operation = history
        .pop_operation()
        .ok_or_else(|| anyhow!("Keine Umbenennung zum Rueckgaengig machen vorhanden"))?;

    let directory = operation.directory.clone();
    let mut undone_count = 0;
    let mut errors = Vec::new();

    // Validate all undo operations first
    for entry in &operation.entries {
        let current_path = directory.join(&entry.new_name);
        let original_path = directory.join(&entry.original_name);

        // Check if current (renamed) file exists
        if !current_path.exists() {
            errors.push(format!(
                "Datei existiert nicht mehr: {} (uebersprungen)",
                entry.new_name
            ));
            continue;
        }

        // Check if original name is already taken by another file
        if original_path.exists() && current_path != original_path {
            if current_path.to_string_lossy().to_lowercase()
                != original_path.to_string_lossy().to_lowercase()
            {
                errors.push(format!(
                    "Urspruenglicher Name bereits vergeben: {} (uebersprungen)",
                    entry.original_name
                ));
                continue;
            }
        }
    }

    // Execute undo renames (reverse: new_name -> original_name)
    for entry in &operation.entries {
        let current_path = directory.join(&entry.new_name);
        let original_path = directory.join(&entry.original_name);

        if !current_path.exists() {
            continue; // Skip files that no longer exist
        }

        if original_path.exists() && current_path != original_path {
            if current_path.to_string_lossy().to_lowercase()
                != original_path.to_string_lossy().to_lowercase()
            {
                continue; // Skip if original name is taken
            }
        }

        match std::fs::rename(&current_path, &original_path) {
            Ok(_) => undone_count += 1,
            Err(e) => {
                errors.push(format!(
                    "Fehler beim Rueckgaengig machen von '{}': {}",
                    entry.new_name, e
                ));
            }
        }
    }

    // Save updated history (with operation removed)
    history.save()?;

    if undone_count == 0 && !errors.is_empty() {
        return Err(anyhow!("Undo fehlgeschlagen:\n{}", errors.join("\n")));
    }

    Ok((undone_count, directory))
}

/// Get a preview of what undo would do
pub fn get_undo_preview() -> Result<Option<(String, Vec<(String, String)>)>> {
    let history = RenameHistory::load()?;

    if let Some(operation) = history.last_operation() {
        let entries: Vec<(String, String)> = operation
            .entries
            .iter()
            .map(|e| (e.new_name.clone(), e.original_name.clone()))
            .collect();
        Ok(Some((operation.description.clone(), entries)))
    } else {
        Ok(None)
    }
}

/// Print previews to stdout (for non-interactive mode)
pub fn print_previews(previews: &[RenamePreview]) {
    let changes: Vec<_> = previews.iter().filter(|p| p.will_change).collect();

    if changes.is_empty() {
        println!("Keine Aenderungen.");
        return;
    }

    println!("\nVorschau der Aenderungen:");
    println!("{:-<60}", "");

    for preview in &changes {
        println!("  {} -> {}", preview.original_name, preview.new_name);
    }

    println!("{:-<60}", "");
    println!("{} Datei(en) werden umbenannt.\n", changes.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_file(name: &str) -> FileEntry {
        FileEntry {
            path: PathBuf::from(name),
            name: name.to_string(),
            is_dir: false,
            size: 0,
            modified: None,
            extension: name.rsplit('.').next().unwrap_or("").to_string(),
        }
    }

    #[test]
    fn test_generate_previews_empty_search() {
        let files = vec![make_file("test.txt")];
        let selected = HashSet::new();

        let previews = generate_previews(
            &files,
            &selected,
            "",
            "replacement",
            RenameMode::SearchReplace,
            PrefixAction::Add,
            1,
            1,
            DatePosition::Prefix,
        )
        .unwrap();

        assert_eq!(previews.len(), 1);
        assert!(!previews[0].will_change);
        assert_eq!(previews[0].original_name, "test.txt");
        assert_eq!(previews[0].new_name, "test.txt");
    }

    #[test]
    fn test_generate_previews_with_replacement() {
        let files = vec![make_file("image001.jpg"), make_file("image002.jpg")];
        let selected = HashSet::new();

        let previews = generate_previews(
            &files,
            &selected,
            "image",
            "photo",
            RenameMode::SearchReplace,
            PrefixAction::Add,
            1,
            1,
            DatePosition::Prefix,
        )
        .unwrap();

        assert_eq!(previews.len(), 2);
        assert!(previews[0].will_change);
        assert_eq!(previews[0].new_name, "photo001.jpg");
        assert!(previews[1].will_change);
        assert_eq!(previews[1].new_name, "photo002.jpg");
    }

    #[test]
    fn test_uppercase_mode() {
        let files = vec![make_file("test.txt")];
        let selected = HashSet::new();

        let previews = generate_previews(
            &files,
            &selected,
            "",
            "",
            RenameMode::Uppercase,
            PrefixAction::Add,
            1,
            1,
            DatePosition::Prefix,
        )
        .unwrap();

        assert_eq!(previews.len(), 1);
        assert!(previews[0].will_change);
        assert_eq!(previews[0].new_name, "TEST.txt");
    }

    #[test]
    fn test_titlecase_mode() {
        let files = vec![make_file("hello_world.txt")];
        let selected = HashSet::new();

        let previews = generate_previews(
            &files,
            &selected,
            "",
            "",
            RenameMode::TitleCase,
            PrefixAction::Add,
            1,
            1,
            DatePosition::Prefix,
        )
        .unwrap();

        assert_eq!(previews.len(), 1);
        assert!(previews[0].will_change);
        assert_eq!(previews[0].new_name, "Hello_World.txt");
    }

    #[test]
    fn test_regex_mode() {
        let files = vec![make_file("IMG_001.jpg"), make_file("IMG_002.jpg")];
        let selected = HashSet::new();

        let previews = generate_previews(
            &files,
            &selected,
            r"IMG_(\d+)",
            "photo_$1",
            RenameMode::Regex,
            PrefixAction::Add,
            1,
            1,
            DatePosition::Prefix,
        )
        .unwrap();

        assert_eq!(previews.len(), 2);
        assert!(previews[0].will_change);
        assert_eq!(previews[0].new_name, "photo_001.jpg");
        assert!(previews[1].will_change);
        assert_eq!(previews[1].new_name, "photo_002.jpg");
    }

    #[test]
    fn test_regex_invalid() {
        let files = vec![make_file("test.txt")];
        let selected = HashSet::new();

        let result = generate_previews(
            &files,
            &selected,
            r"[invalid",
            "replace",
            RenameMode::Regex,
            PrefixAction::Add,
            1,
            1,
            DatePosition::Prefix,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_numbering_mode() {
        let files = vec![make_file("a.jpg"), make_file("b.jpg"), make_file("c.jpg")];
        let mut selected = HashSet::new();
        selected.insert(0);
        selected.insert(1);
        selected.insert(2);

        let previews = generate_previews(
            &files,
            &selected,
            "photo_###",
            "",
            RenameMode::Numbering,
            PrefixAction::Add,
            1,
            1,
            DatePosition::Prefix,
        )
        .unwrap();

        assert_eq!(previews.len(), 3);
        // Note: sorted by original name for display
        assert_eq!(previews[0].new_name, "photo_001.jpg");
        assert_eq!(previews[1].new_name, "photo_002.jpg");
        assert_eq!(previews[2].new_name, "photo_003.jpg");
    }

    #[test]
    fn test_numbering_with_padding() {
        let result = apply_numbering("test.jpg", "file_####", 42);
        assert_eq!(result, "file_0042.jpg");

        let result = apply_numbering("test.jpg", "img#", 5);
        assert_eq!(result, "img5.jpg");
    }

    #[test]
    fn test_prefix_add() {
        let files = vec![make_file("photo.jpg")];
        let selected = HashSet::new();

        let previews = generate_previews(
            &files,
            &selected,
            "backup_",
            "",
            RenameMode::Prefix,
            PrefixAction::Add,
            1,
            1,
            DatePosition::Prefix,
        )
        .unwrap();

        assert_eq!(previews[0].new_name, "backup_photo.jpg");
    }

    #[test]
    fn test_prefix_remove() {
        let files = vec![make_file("backup_photo.jpg")];
        let selected = HashSet::new();

        let previews = generate_previews(
            &files,
            &selected,
            "backup_",
            "",
            RenameMode::Prefix,
            PrefixAction::Remove,
            1,
            1,
            DatePosition::Prefix,
        )
        .unwrap();

        assert_eq!(previews[0].new_name, "photo.jpg");
    }

    #[test]
    fn test_suffix_add() {
        let files = vec![make_file("photo.jpg")];
        let selected = HashSet::new();

        let previews = generate_previews(
            &files,
            &selected,
            "_backup",
            "",
            RenameMode::Suffix,
            PrefixAction::Add,
            1,
            1,
            DatePosition::Prefix,
        )
        .unwrap();

        assert_eq!(previews[0].new_name, "photo_backup.jpg");
    }

    #[test]
    fn test_suffix_remove() {
        let files = vec![make_file("photo_old.jpg")];
        let selected = HashSet::new();

        let previews = generate_previews(
            &files,
            &selected,
            "_old",
            "",
            RenameMode::Suffix,
            PrefixAction::Remove,
            1,
            1,
            DatePosition::Prefix,
        )
        .unwrap();

        assert_eq!(previews[0].new_name, "photo.jpg");
    }

    #[test]
    fn test_date_insert_prefix() {
        use std::time::{Duration, UNIX_EPOCH};

        // November 30, 2024 = days since epoch
        let days = 20058; // Approximate days to Nov 30, 2024
        let time = UNIX_EPOCH + Duration::from_secs(days * 86400);

        let result = apply_date_insert("photo.jpg", DatePosition::Prefix, Some(time));
        assert!(result.starts_with("2024"));
        assert!(result.ends_with("_photo.jpg"));
    }

    #[test]
    fn test_date_insert_suffix() {
        use std::time::{Duration, UNIX_EPOCH};

        let days = 20058;
        let time = UNIX_EPOCH + Duration::from_secs(days * 86400);

        let result = apply_date_insert("photo.jpg", DatePosition::Suffix, Some(time));
        assert!(result.starts_with("photo_"));
        assert!(result.contains("2024"));
        assert!(result.ends_with(".jpg"));
    }

    #[test]
    fn test_date_insert_replace() {
        use std::time::{Duration, UNIX_EPOCH};

        let days = 20058;
        let time = UNIX_EPOCH + Duration::from_secs(days * 86400);

        let result = apply_date_insert("photo.jpg", DatePosition::Replace, Some(time));
        assert!(result.starts_with("2024"));
        assert!(result.ends_with(".jpg"));
        assert!(!result.contains("photo"));
    }

    #[test]
    fn test_date_insert_no_date() {
        let result = apply_date_insert("photo.jpg", DatePosition::Prefix, None);
        assert_eq!(result, "00000000_photo.jpg");
    }
}
