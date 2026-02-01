//! Contains all functionality pertaining to modifying the configuration file for `fetters`.

use std::fs::{File, create_dir_all, read_to_string};
use std::io::Write;
use std::path::PathBuf;

use chrono::Local;
use directories::ProjectDirs;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use toml;

use crate::errors::FettersError;

/// Contains all configuration settings that will be stored in `fetters.toml`.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// The path to the configuration file.
    pub config_path: PathBuf,
    /// The current job sprint.
    pub current_sprint: String,
    /// The path to the SQLite database.
    pub db_path: String,
}

impl Config {
    /// Load the current config file, or create a new one if it doesn't already exist.
    pub fn load_or_create() -> Result<Config, FettersError> {
        let config_path = Self::get_config_dir_path()?.join("fetters.toml");

        if !config_path.exists() {
            println!(
                "{}",
                format!(
                    "ℹ️  Config file not found, creating default at {:?}",
                    config_path
                )
                .cyan()
            );

            Self::create_default_config(&config_path)?;

            let config = Config {
                config_path: config_path.clone(),
                current_sprint: Local::now().date_naive().format("%Y-%m-%d").to_string(),
                db_path: Self::get_data_dir_path()?
                    .join("fetters.db")
                    .to_string_lossy()
                    .into_owned(),
            };
            config.save_to_file()?;

            Ok(config)
        } else {
            let content = read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;

            Ok(config)
        }
    }

    /// Get the project config directory path.
    pub fn get_config_dir_path() -> Result<PathBuf, FettersError> {
        if let Some(ref project_directory) = ProjectDirs::from("", "", "fetters") {
            return Ok(project_directory.config_dir().to_owned());
        }

        Err(FettersError::ApplicationError)
    }

    /// Get the project data directory path.
    fn get_data_dir_path() -> Result<PathBuf, FettersError> {
        if let Some(ref project_directory) = ProjectDirs::from("", "", "fetters") {
            return Ok(project_directory.data_dir().to_owned());
        }

        Err(FettersError::ApplicationError)
    }

    /// Create the `fetters.toml` file.
    fn create_default_config(config_path: &PathBuf) -> Result<(), FettersError> {
        if let Some(parent) = config_path.parent() {
            create_dir_all(parent)?;
        }

        File::create(config_path)?;

        Ok(())
    }

    /// Save the current config to the `fetters.toml` file by overwriting it.
    pub fn save_to_file(&self) -> Result<(), FettersError> {
        let toml_str = toml::to_string_pretty(&self)?;
        let mut file = File::create(self.config_path.clone())?;
        file.write_all(toml_str.as_bytes())?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_get_config_dir_path_returns_path() {
        let path = Config::get_config_dir_path();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("fetters"));
    }

    #[test]
    fn test_save_and_load_config_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("fetters.toml");

        let config = Config {
            config_path: config_path.clone(),
            current_sprint: "2025-01-15".to_string(),
            db_path: "/tmp/test.db".to_string(),
        };
        config.save_to_file().unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        let loaded: Config = toml::from_str(&content).unwrap();
        assert_eq!(loaded.current_sprint, "2025-01-15");
        assert_eq!(loaded.db_path, "/tmp/test.db");
    }

    #[test]
    fn test_create_default_config_creates_file_and_parents() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("subdir").join("fetters.toml");

        Config::create_default_config(&config_path).unwrap();
        assert!(config_path.exists());
    }

    #[test]
    fn test_save_to_file_overwrites_existing() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("fetters.toml");

        let config1 = Config {
            config_path: config_path.clone(),
            current_sprint: "sprint-1".to_string(),
            db_path: "/tmp/test.db".to_string(),
        };
        config1.save_to_file().unwrap();

        let config2 = Config {
            config_path: config_path.clone(),
            current_sprint: "sprint-2".to_string(),
            db_path: "/tmp/test.db".to_string(),
        };
        config2.save_to_file().unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        let loaded: Config = toml::from_str(&content).unwrap();
        assert_eq!(loaded.current_sprint, "sprint-2");
    }
}
