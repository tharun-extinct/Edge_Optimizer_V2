use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration storing current state
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    /// Name of currently active profile (None = inactive)
    pub active_profile: Option<String>,
    /// Whether overlay is currently visible
    pub overlay_visible: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            active_profile: None,
            overlay_visible: false,
        }
    }
}

/// Get the application's data directory
/// Returns %APPDATA%/GamingOptimizer/ on Windows
/// Creates directory if it doesn't exist
pub fn get_data_directory() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("", "", "GamingOptimizer")
        .ok_or_else(|| anyhow!("Failed to determine user data directory"))?;

    let data_dir = project_dirs.data_dir();

    // Create directory if it doesn't exist
    fs::create_dir_all(data_dir)
        .map_err(|e| anyhow!("Failed to create data directory: {}", e))?;

    Ok(data_dir.to_path_buf())
}

/// Load application configuration from config.json
/// Returns default config if file doesn't exist
pub fn load_config() -> Result<AppConfig> {
    let data_dir = get_data_directory()?;
    let config_path = data_dir.join("config.json");

    // If file doesn't exist, return default config
    if !config_path.exists() {
        return Ok(AppConfig::default());
    }

    // Read and parse JSON
    let contents = fs::read_to_string(&config_path)
        .map_err(|e| anyhow!("Failed to read config.json: {}", e))?;

    let config: AppConfig = serde_json::from_str(&contents)
        .map_err(|e| anyhow!("Failed to parse config.json: {}", e))?;

    Ok(config)
}

/// Save application configuration to config.json
pub fn save_config(config: &AppConfig) -> Result<()> {
    let data_dir = get_data_directory()?;
    let config_path = data_dir.join("config.json");

    // Serialize to pretty-printed JSON
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| anyhow!("Failed to serialize config: {}", e))?;

    // Write to file
    fs::write(&config_path, json)
        .map_err(|e| anyhow!("Failed to write config.json: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.active_profile, None);
        assert_eq!(config.overlay_visible, false);
    }

    #[test]
    fn test_get_data_directory() {
        let result = get_data_directory();
        assert!(result.is_ok());

        let path = result.unwrap();
        assert!(path.to_string_lossy().contains("GamingOptimizer"));
    }
}
