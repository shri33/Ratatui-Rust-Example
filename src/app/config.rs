use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme_index: usize,
    pub default_tab: usize,
    pub auto_save: bool,
    pub themes: Vec<Theme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub primary_color: String,
    pub secondary_color: String,
    pub background_color: String,
    pub text_color: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme_index: 0,
            default_tab: 0,
            auto_save: true,
            themes: vec![
                Theme {
                    name: "Default".into(),
                    primary_color: "#FF0000".into(),
                    secondary_color: "#00FF00".into(),
                    background_color: "#0000FF".into(),
                    text_color: "#FFFFFF".into(),
                },
                Theme {
                    name: "Solarized".into(),
                    primary_color: "#268986".into(),
                    secondary_color: "#b58900".into(),
                    background_color: "#fdf6e3".into(),
                    text_color: "#657b83".into(),
                },
            ],
        }
    }
}

impl Config {
    /// Load configuration from file or create default
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let config_path = Self::config_path()?;
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), Box<dyn Error>> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&config_path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf, Box<dyn Error>> {
        let mut path = dirs::config_dir().ok_or("Could not find config directory")?;
        path.push("ratatui-rust-example");
        path.push("config.toml");
        Ok(path)
    }

    /// Get the current theme
    pub fn current_theme(&self) -> &Theme {
        &self.themes[self.theme_index]
    }
}
