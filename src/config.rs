use crate::models::Config;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Load configuration from file or use defaults
///
/// Search order:
/// 1. Custom path if provided via --config
/// 2. .fossilrc in current directory
/// 3. ~/.fossilrc in home directory
/// 4. Built-in defaults
pub fn load_config(custom_path: Option<&Path>) -> Result<Config> {
    // If custom path provided, use it exclusively
    if let Some(path) = custom_path {
        return load_config_from_file(path)
            .with_context(|| format!("Failed to load config from {}", path.display()));
    }

    // Try current directory
    let current_config = PathBuf::from(".fossilrc");
    if current_config.exists() {
        if let Ok(config) = load_config_from_file(&current_config) {
            return Ok(config);
        }
    }

    // Try home directory
    if let Some(home_config) = get_home_config_path() {
        if home_config.exists() {
            if let Ok(config) = load_config_from_file(&home_config) {
                return Ok(config);
            }
        }
    }

    // Fall back to defaults
    Ok(Config::default())
}

/// Load config from a specific file
fn load_config_from_file(path: &Path) -> Result<Config> {
    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;

    let config: Config = toml::from_str(&contents)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

    Ok(config)
}

/// Get path to home directory config file
fn get_home_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".fossilrc"))
}

/// Save a config to a file (useful for creating example configs)
pub fn save_config(config: &Config, path: &Path) -> Result<()> {
    let toml_string = toml::to_string_pretty(config)
        .context("Failed to serialize config")?;

    fs::write(path, toml_string)
        .with_context(|| format!("Failed to write config to {}", path.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_load_default_config() {
        let config = load_config(None).unwrap();
        assert!(!config.markers.is_empty());
        assert!(config.markers.contains(&"TODO".to_string()));
    }

    #[test]
    fn test_load_custom_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
markers = ["TODO", "FIXME", "CUSTOM"]
ignored_dirs = [".git", "custom_ignore"]
context_lines = 3

[severity]
FIXME = "high"
TODO = "low"
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let config = load_config_from_file(temp_file.path()).unwrap();
        assert_eq!(config.markers.len(), 3);
        assert!(config.markers.contains(&"CUSTOM".to_string()));
        assert_eq!(config.context_lines, 3);
        assert_eq!(config.severity.get("FIXME"), Some(&"high".to_string()));
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = Config::default();

        save_config(&config, temp_file.path()).unwrap();
        let loaded = load_config_from_file(temp_file.path()).unwrap();

        assert_eq!(config.markers, loaded.markers);
        assert_eq!(config.context_lines, loaded.context_lines);
    }
}
