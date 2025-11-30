use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::app::{DatePosition, RenameMode, SortOrder};

/// A saved rename preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    /// Name of the preset
    pub name: String,
    /// Rename mode
    pub mode: RenameMode,
    /// Search pattern (for SearchReplace mode)
    #[serde(default)]
    pub search: String,
    /// Replace pattern (for SearchReplace mode)
    #[serde(default)]
    pub replace: String,
}

impl Preset {
    pub fn new(name: String, mode: RenameMode, search: String, replace: String) -> Self {
        Self {
            name,
            mode,
            search,
            replace,
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default rename mode
    #[serde(default)]
    pub default_mode: RenameMode,
    
    /// Default sort order
    #[serde(default)]
    pub default_sort: SortOrder,
    
    /// Saved presets
    #[serde(default)]
    pub presets: HashMap<String, Preset>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_mode: RenameMode::SearchReplace,
            default_sort: SortOrder::Name,
            presets: HashMap::new(),
        }
    }
}

impl Config {
    /// Get the config file path
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("rnm").join("config.toml"))
    }

    /// Load config from file, or return default if file doesn't exist
    pub fn load() -> Result<Self> {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Ok(Self::default()),
        };

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Konnte Konfiguration nicht lesen: {}", path.display()))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Ungueltige Konfiguration: {}", path.display()))?;

        Ok(config)
    }

    /// Save config to file
    pub fn save(&self) -> Result<()> {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Ok(()),
        };

        // Create config directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Konnte Verzeichnis nicht erstellen: {}", parent.display()))?;
        }

        let content = toml::to_string_pretty(self)
            .context("Konnte Konfiguration nicht serialisieren")?;

        fs::write(&path, content)
            .with_context(|| format!("Konnte Konfiguration nicht schreiben: {}", path.display()))?;

        Ok(())
    }

    /// Add or update a preset
    pub fn add_preset(&mut self, preset: Preset) {
        self.presets.insert(preset.name.clone(), preset);
    }

    /// Remove a preset
    pub fn remove_preset(&mut self, name: &str) -> Option<Preset> {
        self.presets.remove(name)
    }

    /// Get a preset by name
    pub fn get_preset(&self, name: &str) -> Option<&Preset> {
        self.presets.get(name)
    }

    /// List all preset names
    pub fn list_presets(&self) -> Vec<&str> {
        self.presets.keys().map(|s| s.as_str()).collect()
    }
}

/// Parse mode string from CLI argument
pub fn parse_mode(mode_str: &str) -> Option<RenameMode> {
    match mode_str.to_lowercase().as_str() {
        "search" | "searchreplace" | "search-replace" | "s" => Some(RenameMode::SearchReplace),
        "regex" | "r" => Some(RenameMode::Regex),
        "numbering" | "number" | "num" | "n" => Some(RenameMode::Numbering),
        "prefix" | "pre" => Some(RenameMode::Prefix),
        "suffix" | "suf" => Some(RenameMode::Suffix),
        "date" | "dateinsert" | "date-insert" | "d" => Some(RenameMode::DateInsert),
        "upper" | "uppercase" | "u" => Some(RenameMode::Uppercase),
        "lower" | "lowercase" | "l" => Some(RenameMode::Lowercase),
        "title" | "titlecase" | "t" => Some(RenameMode::TitleCase),
        _ => None,
    }
}

/// Parse date position string from CLI argument
pub fn parse_date_position(position_str: &str) -> Option<DatePosition> {
    match position_str.to_lowercase().as_str() {
        "prefix" | "pre" | "p" => Some(DatePosition::Prefix),
        "suffix" | "suf" | "s" => Some(DatePosition::Suffix),
        "replace" | "rep" | "r" => Some(DatePosition::Replace),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_mode, RenameMode::SearchReplace);
        assert_eq!(config.default_sort, SortOrder::Name);
        assert!(config.presets.is_empty());
    }

    #[test]
    fn test_preset_management() {
        let mut config = Config::default();
        
        let preset = Preset::new(
            "test".to_string(),
            RenameMode::SearchReplace,
            "foo".to_string(),
            "bar".to_string(),
        );
        
        config.add_preset(preset);
        
        assert!(config.get_preset("test").is_some());
        assert_eq!(config.get_preset("test").unwrap().search, "foo");
        
        config.remove_preset("test");
        assert!(config.get_preset("test").is_none());
    }

    #[test]
    fn test_parse_mode() {
        assert_eq!(parse_mode("upper"), Some(RenameMode::Uppercase));
        assert_eq!(parse_mode("UPPER"), Some(RenameMode::Uppercase));
        assert_eq!(parse_mode("lowercase"), Some(RenameMode::Lowercase));
        assert_eq!(parse_mode("title"), Some(RenameMode::TitleCase));
        assert_eq!(parse_mode("search"), Some(RenameMode::SearchReplace));
        assert_eq!(parse_mode("date"), Some(RenameMode::DateInsert));
        assert_eq!(parse_mode("invalid"), None);
    }

    #[test]
    fn test_parse_date_position() {
        assert_eq!(parse_date_position("prefix"), Some(DatePosition::Prefix));
        assert_eq!(parse_date_position("SUFFIX"), Some(DatePosition::Suffix));
        assert_eq!(parse_date_position("replace"), Some(DatePosition::Replace));
        assert_eq!(parse_date_position("p"), Some(DatePosition::Prefix));
        assert_eq!(parse_date_position("invalid"), None);
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::default();
        config.default_mode = RenameMode::Uppercase;
        config.add_preset(Preset::new(
            "my-preset".to_string(),
            RenameMode::SearchReplace,
            "old".to_string(),
            "new".to_string(),
        ));

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(loaded.default_mode, RenameMode::Uppercase);
        assert!(loaded.get_preset("my-preset").is_some());
    }
}

