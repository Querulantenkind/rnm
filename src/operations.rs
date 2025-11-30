use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::app::{FileEntry, RenameMode};

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
) -> Vec<RenamePreview> {
    let mut previews = Vec::new();

    // If nothing is selected, preview all files
    let indices: Vec<usize> = if selected.is_empty() {
        (0..files.len()).collect()
    } else {
        selected.iter().copied().collect()
    };

    for index in indices {
        if let Some(file) = files.get(index) {
            // Skip directories for now
            if file.is_dir {
                continue;
            }

            let new_name = apply_rename_mode(&file.name, search, replace, mode);
            let will_change = new_name != file.name;

            previews.push(RenamePreview {
                original_name: file.name.clone(),
                new_name,
                will_change,
                file_index: index,
            });
        }
    }

    // Sort by original name
    previews.sort_by(|a, b| a.original_name.cmp(&b.original_name));

    previews
}

/// Apply the rename mode to a filename
fn apply_rename_mode(filename: &str, search: &str, replace: &str, mode: RenameMode) -> String {
    match mode {
        RenameMode::SearchReplace => {
            if search.is_empty() {
                filename.to_string()
            } else {
                filename.replace(search, replace)
            }
        }
        RenameMode::Uppercase => to_uppercase_preserve_extension(filename),
        RenameMode::Lowercase => to_lowercase_preserve_extension(filename),
        RenameMode::TitleCase => to_titlecase_preserve_extension(filename),
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

/// Execute the actual rename operations
pub fn execute_renames(previews: &[RenamePreview], directory: &PathBuf) -> Result<usize> {
    let mut renamed_count = 0;
    let mut errors = Vec::new();

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
            if old_path.to_string_lossy().to_lowercase() != new_path.to_string_lossy().to_lowercase() {
                errors.push(format!(
                    "Zieldatei existiert bereits: {}",
                    preview.new_name
                ));
                continue;
            }
        }

        // Check for invalid characters in new name
        if preview.new_name.contains('/') || preview.new_name.contains('\\') {
            errors.push(format!(
                "Ungueltiger Dateiname: {}",
                preview.new_name
            ));
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
            Ok(_) => renamed_count += 1,
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

    Ok(renamed_count)
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

    #[test]
    fn test_generate_previews_empty_search() {
        let files = vec![
            FileEntry {
                path: PathBuf::from("test.txt"),
                name: "test.txt".to_string(),
                is_dir: false,
                size: 0,
                modified: None,
                extension: "txt".to_string(),
            },
        ];
        let selected = HashSet::new();

        let previews = generate_previews(&files, &selected, "", "replacement", RenameMode::SearchReplace);

        assert_eq!(previews.len(), 1);
        assert!(!previews[0].will_change);
        assert_eq!(previews[0].original_name, "test.txt");
        assert_eq!(previews[0].new_name, "test.txt");
    }

    #[test]
    fn test_generate_previews_with_replacement() {
        let files = vec![
            FileEntry {
                path: PathBuf::from("image001.jpg"),
                name: "image001.jpg".to_string(),
                is_dir: false,
                size: 0,
                modified: None,
                extension: "jpg".to_string(),
            },
            FileEntry {
                path: PathBuf::from("image002.jpg"),
                name: "image002.jpg".to_string(),
                is_dir: false,
                size: 0,
                modified: None,
                extension: "jpg".to_string(),
            },
        ];
        let selected = HashSet::new();

        let previews = generate_previews(&files, &selected, "image", "photo", RenameMode::SearchReplace);

        assert_eq!(previews.len(), 2);
        assert!(previews[0].will_change);
        assert_eq!(previews[0].new_name, "photo001.jpg");
        assert!(previews[1].will_change);
        assert_eq!(previews[1].new_name, "photo002.jpg");
    }

    #[test]
    fn test_uppercase_mode() {
        let files = vec![
            FileEntry {
                path: PathBuf::from("test.txt"),
                name: "test.txt".to_string(),
                is_dir: false,
                size: 0,
                modified: None,
                extension: "txt".to_string(),
            },
        ];
        let selected = HashSet::new();

        let previews = generate_previews(&files, &selected, "", "", RenameMode::Uppercase);

        assert_eq!(previews.len(), 1);
        assert!(previews[0].will_change);
        assert_eq!(previews[0].new_name, "TEST.txt");
    }

    #[test]
    fn test_titlecase_mode() {
        let files = vec![
            FileEntry {
                path: PathBuf::from("hello_world.txt"),
                name: "hello_world.txt".to_string(),
                is_dir: false,
                size: 0,
                modified: None,
                extension: "txt".to_string(),
            },
        ];
        let selected = HashSet::new();

        let previews = generate_previews(&files, &selected, "", "", RenameMode::TitleCase);

        assert_eq!(previews.len(), 1);
        assert!(previews[0].will_change);
        assert_eq!(previews[0].new_name, "Hello_World.txt");
    }
}
