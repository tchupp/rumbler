use crate::config::ConfigPath;
use crate::error::RumblerError;
use envconfig::Envconfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Envconfig)]
struct RamblerConfigEnvironment {
    #[envconfig(from = "RAMBLER_DATABASE")]
    pub database: Option<String>,

    #[envconfig(from = "RAMBLER_HOST")]
    pub host: Option<String>,

    #[envconfig(from = "RAMBLER_PORT")]
    pub port: Option<u16>,

    #[envconfig(from = "RAMBLER_USER")]
    pub user: Option<String>,

    #[envconfig(from = "RAMBLER_PASSWORD")]
    pub password: Option<String>,

    #[envconfig(from = "RAMBLER_ROLE")]
    pub role: Option<String>,

    #[envconfig(from = "RAMBLER_SCHEMA")]
    pub schema: Option<String>,

    #[envconfig(from = "RAMBLER_SSLMODE")]
    pub sslmode: Option<String>,

    #[envconfig(from = "RAMBLER_DIRECTORY")]
    pub directory: Option<String>,

    #[envconfig(from = "RAMBLER_TABLE")]
    pub table: Option<String>,
}

#[derive(Debug, Clone, Envconfig)]
struct RumblerConfigEnvironment {
    #[envconfig(from = "RUMBLER_DATABASE")]
    pub database: Option<String>,

    #[envconfig(from = "RUMBLER_HOST")]
    pub host: Option<String>,

    #[envconfig(from = "RUMBLER_PORT")]
    pub port: Option<u16>,

    #[envconfig(from = "RUMBLER_USER")]
    pub user: Option<String>,

    #[envconfig(from = "RUMBLER_PASSWORD")]
    pub password: Option<String>,

    #[envconfig(from = "RUMBLER_ROLE")]
    pub role: Option<String>,

    #[envconfig(from = "RUMBLER_SCHEMA")]
    pub schema: Option<String>,

    #[envconfig(from = "RUMBLER_SEARCH_PATH")]
    pub search_path: Option<String>,

    #[envconfig(from = "RUMBLER_SSLMODE")]
    pub sslmode: Option<String>,

    #[envconfig(from = "RUMBLER_DIRECTORY")]
    pub directory: Option<String>,

    #[envconfig(from = "RUMBLER_TABLE")]
    pub table: Option<String>,
}

#[derive(Default, Debug, Clone, Deserialize)]
pub struct PartialConfig {
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub user: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub schema: Option<String>,
    #[serde(default)]
    pub search_path: Option<String>,
    #[serde(default)]
    pub sslmode: Option<String>,
    #[serde(default)]
    pub directory: Option<String>,
    #[serde(default)]
    pub table: Option<String>,
    #[serde(default)]
    pub environments: HashMap<String, PartialConfig>,
}

macro_rules! or {
    ($field:ident, $first_option:ident, $($option:ident),+ $(,)?) => {
        $first_option.$field
        $( .or($option.$field) )+
        .or_else(|| None)
    };
}

pub fn try_load_config(
    config_path: Option<impl Into<String>>,
) -> Result<PartialConfig, RumblerError> {
    let rumbler_config = RumblerConfigEnvironment::init_from_env()
        .map_err(|e| RumblerError::ConfigParse(format!("env var error: {e}")))?;
    let rambler_config = RamblerConfigEnvironment::init_from_env()
        .map_err(|e| RumblerError::ConfigParse(format!("env var error: {e}")))?;
    let file_config = resolve_file_config(config_path)?;

    Ok(PartialConfig {
        database: or!(database, rumbler_config, rambler_config, file_config),
        host: or!(host, rumbler_config, rambler_config, file_config),
        port: or!(port, rumbler_config, rambler_config, file_config),
        user: or!(user, rumbler_config, rambler_config, file_config),
        password: or!(password, rumbler_config, rambler_config, file_config),
        role: or!(role, rumbler_config, rambler_config, file_config),
        schema: or!(schema, rumbler_config, rambler_config, file_config),
        search_path: or!(search_path, rumbler_config, file_config),
        sslmode: or!(sslmode, rumbler_config, rambler_config, file_config),
        directory: or!(directory, rumbler_config, rambler_config, file_config),
        table: or!(table, rumbler_config, rambler_config, file_config),
        environments: file_config.environments,
    })
}

fn resolve_file_config(
    config_path: Option<impl Into<String>>,
) -> Result<PartialConfig, RumblerError> {
    let config_path = resolve_config_path(config_path)?;
    let file: PartialConfig = match config_path {
        ConfigPath::Toml(path) => {
            let contents = std::fs::read_to_string(&path)?;
            toml::from_str(&contents)
                .map_err(|e| RumblerError::ConfigParse(format!("{path:?}: {e}")))?
        }
        ConfigPath::Json(path) => {
            let contents = std::fs::read_to_string(&path)?;
            serde_json::from_str(&contents)
                .map_err(|e| RumblerError::ConfigParse(format!("{path:?}: {e}")))?
        }
        ConfigPath::None => PartialConfig::default(),
    };
    Ok(file)
}

fn resolve_config_path(config_path: Option<impl Into<String>>) -> Result<ConfigPath, RumblerError> {
    if let Some(config_path) = config_path {
        let config_path = config_path.into();
        let path = Path::new(config_path.as_str());
        return if path.exists() {
            if path.extension().is_some_and(|ext| ext == "json") {
                Ok(ConfigPath::Json(path.to_path_buf()))
            } else if path.extension().is_some_and(|ext| ext == "toml") {
                Ok(ConfigPath::Toml(path.to_path_buf()))
            } else {
                Err(RumblerError::ConfigParse(format!(
                    "unsupported config file extension: {:?}",
                    path.extension()
                )))
            }
        } else {
            Err(RumblerError::ConfigNotFound(path.to_path_buf()))
        };
    }

    let path = Path::new("rumbler.toml");
    if path.exists() {
        return Ok(ConfigPath::Toml(path.to_path_buf()));
    }

    let path = Path::new("rambler.json");
    if path.exists() {
        return Ok(ConfigPath::Json(path.to_path_buf()));
    }

    Ok(ConfigPath::None)
}
