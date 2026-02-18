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

    #[serde(default)]
    pub font: FontConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_true")]
    pub sidebar_open_on_start: bool,

    #[serde(default = "default_true")]
    pub enable_animations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default = "default_window_width")]
    pub width: f32,

    #[serde(default = "default_window_height")]
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemeConfig {
    #[default]
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    #[serde(default = "default_font_family")]
    pub family: FontFamily,

    #[serde(default = "default_font_size")]
    pub size: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FontFamily {
    JetBrainsMono,
    FiraCode,
    SourceCodePro,
    DejaVuSansMono,
    CourierNew,
}

// Default value functions for serde
fn default_theme() -> ThemeConfig {
    ThemeConfig::Light
}

fn default_font_family() -> FontFamily {
    FontFamily::JetBrainsMono
}

fn default_font_size() -> f32 {
    14.0
}

fn default_true() -> bool {
    true
}

fn default_window_width() -> f32 {
    800.0
}

fn default_window_height() -> f32 {
    600.0
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            font: FontConfig::default(),
        }
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: default_font_family(),
            size: default_font_size(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            sidebar_open_on_start: default_true(),
            enable_animations: default_true(),
        }
    }
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: default_window_width(),
            height: default_window_height(),
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
    /// SELF-HEALING: Automatically rewrites the config file to:
    ///   - Add any missing fields with defaults
    ///   - Remove obsolete fields not in current version
    ///   - Ensure file matches current structure
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
                    match toml::from_str::<Config>(&contents) {
                        Ok(config) => {
                            println!("Loaded config from {:?}", config_path);

                            // SELF-HEALING: Rewrite config if it differs from current structure
                            // This automatically:
                            // 1. Adds any missing fields that were filled by #[serde(default)]
                            // 2. Removes obsolete fields (by serializing only current struct)
                            // 3. Ensures file matches current code structure

                            if let Ok(current_serialized) = toml::to_string_pretty(&config) {
                                if current_serialized.trim() != contents.trim() {
                                    println!("Healing config: adding missing fields and removing obsolete fields");

                                    if let Err(e) = config.save() {
                                        eprintln!("Failed to save healed config: {}", e);
                                    } else {
                                        println!("Config healed successfully");
                                    }
                                }
                            }

                            return config;
                        }
                        Err(e) => {
                            eprintln!("Failed to parse config file: {}", e);
                            eprintln!("The config file may be corrupted.");
                            eprintln!("Creating backup and using default configuration");

                            // Try to backup the old config
                            let backup_path = config_path.with_extension("toml.backup");
                            if let Err(e) = fs::copy(&config_path, &backup_path) {
                                eprintln!("Failed to backup old config: {}", e);
                            } else {
                                println!("Old config backed up to {:?}", backup_path);
                            }

                            // Create and save new default config
                            let default_config = Self::default();
                            if let Err(e) = default_config.save() {
                                eprintln!("Failed to save default config: {}", e);
                            }
                            return default_config;
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

    /// Update font family and save
    pub fn set_font_family(&mut self, family: FontFamily) -> Result<(), ConfigError> {
        self.appearance.font.family = family;
        self.save()
    }

    /// Update font size and save
    pub fn set_font_size(&mut self, size: f32) -> Result<(), ConfigError> {
        self.appearance.font.size = size;
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

impl FontFamily {
    pub fn all() -> &'static [FontFamily] {
        &[
            FontFamily::JetBrainsMono,
            FontFamily::FiraCode,
            FontFamily::SourceCodePro,
            FontFamily::DejaVuSansMono,
            FontFamily::CourierNew,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            FontFamily::JetBrainsMono => "JetBrains Mono",
            FontFamily::FiraCode => "Fira Code",
            FontFamily::SourceCodePro => "Source Code Pro",
            FontFamily::DejaVuSansMono => "DejaVu Sans Mono",
            FontFamily::CourierNew => "Courier New",
        }
    }

    /// Convert to iced::Font
    /// Maps each font family to an iced::Font
    /// Note: System needs to have these fonts installed for them to work
    pub fn to_iced_font(&self) -> iced::Font {
        match self {
            FontFamily::JetBrainsMono => iced::Font {
                family: iced::font::Family::Name("JetBrains Mono"),
                weight: iced::font::Weight::Normal,
                ..iced::Font::MONOSPACE
            },
            FontFamily::FiraCode => iced::Font {
                family: iced::font::Family::Name("Fira Code"),
                weight: iced::font::Weight::Normal,
                ..iced::Font::MONOSPACE
            },
            FontFamily::SourceCodePro => iced::Font {
                family: iced::font::Family::Name("Source Code Pro"),
                weight: iced::font::Weight::Normal,
                ..iced::Font::MONOSPACE
            },
            FontFamily::DejaVuSansMono => iced::Font {
                family: iced::font::Family::Name("DejaVu Sans Mono"),
                weight: iced::font::Weight::Normal,
                ..iced::Font::MONOSPACE
            },
            FontFamily::CourierNew => iced::Font {
                family: iced::font::Family::Name("Courier New"),
                weight: iced::font::Weight::Normal,
                ..iced::Font::MONOSPACE
            },
        }
    }
}

impl std::fmt::Display for FontFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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

