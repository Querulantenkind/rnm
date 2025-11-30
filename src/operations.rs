use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{anyhow, Result};

use crate::app::FileEntry;

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

/// Generate previews for all selected files based on search/replace
pub fn generate_previews(
    files: &[FileEntry],
    selected: &HashSet<usize>,
    search: &str,
    replace: &str,
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

            let new_name = if search.is_empty() {
                file.name.clone()
            } else {
                file.name.replace(search, replace)
            };

            let will_change = new_name != file.name && !search.is_empty();

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
            errors.push(format!(
                "Zieldatei existiert bereits: {}",
                preview.new_name
            ));
            continue;
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
            },
        ];
        let selected = HashSet::new();

        let previews = generate_previews(&files, &selected, "", "replacement");

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
            },
            FileEntry {
                path: PathBuf::from("image002.jpg"),
                name: "image002.jpg".to_string(),
                is_dir: false,
            },
        ];
        let selected = HashSet::new();

        let previews = generate_previews(&files, &selected, "image", "photo");

        assert_eq!(previews.len(), 2);
        assert!(previews[0].will_change);
        assert_eq!(previews[0].new_name, "photo001.jpg");
        assert!(previews[1].will_change);
        assert_eq!(previews[1].new_name, "photo002.jpg");
    }
}

