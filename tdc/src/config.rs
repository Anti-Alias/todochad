use std::{fs, env};
use std::path::PathBuf;
use ron::ser::PrettyConfig;
use thiserror::Error;
use serde::{Serialize, Deserialize};

const APP_DIR_NAME: &str        = "tdc";
const CONFIG_FILE_NAME: &str    = "config.ron";
const GRAPH_FILE_NAME: &str     = "graph.ron";

/// TodoChad application configuration
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct Config { pub graph_path: PathBuf }
impl Config {

    /// Loads the config file from its standard location, creating it if it does not exist.
    pub fn load() -> Result<Self> {
        let config_path = config_path()?;
        if let Some(config_dir) = config_path.parent() {
            fs::create_dir_all(config_dir)?;
        }
        match fs::exists(&config_path) {
            Ok(true) => {
                let config_string = std::fs::read_to_string(&config_path)?;
                ron::de::from_str(&config_string).map_err(|_| ConfigError::ConfigParseError)
            },
            Ok(false) => {
                let graph_path = default_graph_path()?;
                let config = Config { graph_path };
                let config_string = ron::ser::to_string_pretty(&config, PrettyConfig::default()).expect("Failed to serialize config file");
                fs::write(&config_path, config_string)?;
                Ok(config)
            },
            Err(_) => todo!(),
        }
    }

}

fn config_path() -> Result<PathBuf> {
    let home = env::var("HOME").map_err(|_| ConfigError::HomeDirError)?;
    let config_path = format!("{home}/.config/{APP_DIR_NAME}/{CONFIG_FILE_NAME}");
    Ok(PathBuf::from(config_path))
}

fn default_graph_path() -> Result<PathBuf> {
    let home = env::var("HOME").map_err(|_| ConfigError::HomeDirError)?;
    let config_path = format!("{home}/.local/share/{APP_DIR_NAME}/{GRAPH_FILE_NAME}");
    Ok(PathBuf::from(config_path))
}


#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to get home directory")]
    HomeDirError,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Failed to parse config file")]
    ConfigParseError,
}

type Result<T> = std::result::Result<T, ConfigError>;
