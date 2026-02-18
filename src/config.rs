use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const CONFIG_DIR_NAME: &str = "cards";
const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub appearance: AppearanceConfig,

    #[serde(default)]
    pub general: GeneralConfig,

    #[serde(default)]
    pub window: WindowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    #[serde(default = "default_theme")]
    pub theme: ThemeConfig,

    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: f32,

    #[serde(default = "default_dot_spacing")]
    pub dot_spacing: f32,

    #[serde(default = "default_dot_radius")]
    pub dot_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_true")]
    pub sidebar_open_on_start: bool,

    #[serde(default = "default_true")]
    pub enable_animations: bool,

    #[serde(default = "default_animation_duration")]
    pub animation_duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_window_width")]
    pub width: f32,

    #[serde(default = "default_window_height")]
    pub height: f32,

    #[serde(default = "default_min_window_width")]
    pub min_width: f32,

    #[serde(default = "default_min_window_height")]
    pub min_height: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeConfig {
    #[default]
    Light,
    Dark,
}

// Default value functions for serde
fn default_theme() -> ThemeConfig {
    ThemeConfig::Light
}

fn default_sidebar_width() -> f32 {
    250.0
}

fn default_dot_spacing() -> f32 {
    30.0
}

fn default_dot_radius() -> f32 {
    2.0
}

fn default_true() -> bool {
    true
}

fn default_animation_duration() -> u32 {
    250
}

fn default_window_width() -> f32 {
    800.0
}

fn default_window_height() -> f32 {
    600.0
}

fn default_min_window_width() -> f32 {
    800.0
}

fn default_min_window_height() -> f32 {
    600.0
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            sidebar_width: default_sidebar_width(),
            dot_spacing: default_dot_spacing(),
            dot_radius: default_dot_radius(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            sidebar_open_on_start: default_true(),
            enable_animations: default_true(),
            animation_duration_ms: default_animation_duration(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: default_window_width(),
            height: default_window_height(),
            min_width: default_min_window_width(),
            min_height: default_min_window_height(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            appearance: AppearanceConfig::default(),
            general: GeneralConfig::default(),
            window: WindowConfig::default(),
        }
    }
}

impl Config {
    /// Get the configuration directory path
    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join(CONFIG_DIR_NAME))
    }

    /// Get the configuration file path
    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join(CONFIG_FILE_NAME))
    }

    /// Load configuration from file, creating default if it doesn't exist
    pub fn load() -> Self {
        let config_path = match Self::config_path() {
            Some(path) => path,
            None => {
                eprintln!("Could not determine config directory, using defaults");
                return Self::default();
            }
        };

        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(contents) => {
                    match toml::from_str(&contents) {
                        Ok(config) => {
                            println!("Loaded config from {:?}", config_path);
                            return config;
                        }
                        Err(e) => {
                            eprintln!("Failed to parse config file: {}", e);
                            eprintln!("Using default configuration");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to read config file: {}", e);
                    eprintln!("Using default configuration");
                }
            }
        } else {
            println!("Config file not found, creating default at {:?}", config_path);
            let default_config = Self::default();
            if let Err(e) = default_config.save() {
                eprintln!("Failed to save default config: {}", e);
            }
            return default_config;
        }

        Self::default()
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        let config_dir = Self::config_dir()
            .ok_or(ConfigError::NoConfigDir)?;

        let config_path = Self::config_path()
            .ok_or(ConfigError::NoConfigDir)?;

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
        }

        // Serialize config to TOML
        let toml_string = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;

        // Write to file
        let mut file = fs::File::create(&config_path)
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        file.write_all(toml_string.as_bytes())
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        println!("Saved config to {:?}", config_path);
        Ok(())
    }

    /// Update theme and save
    pub fn set_theme(&mut self, theme: Theme) -> Result<(), ConfigError> {
        self.appearance.theme = theme.into();
        self.save()
    }

    /// Update sidebar open on start and save
    pub fn set_sidebar_open_on_start(&mut self, open: bool) -> Result<(), ConfigError> {
        self.general.sidebar_open_on_start = open;
        self.save()
    }

    /// Update animations enabled and save
    pub fn set_animations_enabled(&mut self, enabled: bool) -> Result<(), ConfigError> {
        self.general.enable_animations = enabled;
        self.save()
    }
}

impl From<ThemeConfig> for Theme {
    fn from(config: ThemeConfig) -> Self {
        match config {
            ThemeConfig::Light => Theme::Light,
            ThemeConfig::Dark => Theme::Dark,
        }
    }
}

impl From<Theme> for ThemeConfig {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Light => ThemeConfig::Light,
            Theme::Dark => ThemeConfig::Dark,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfigError {
    NoConfigDir,
    IoError(String),
    SerializeError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NoConfigDir => write!(f, "Could not determine config directory"),
            ConfigError::IoError(e) => write!(f, "IO error: {}", e),
            ConfigError::SerializeError(e) => write!(f, "Serialization error: {}", e)
        }
    }
}

impl std::error::Error for ConfigError {}

