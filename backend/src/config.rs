use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use log::LevelFilter;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub database_url: String,
    #[serde(default = "default_create_database")]
    pub create_database: bool,
    pub health_check_port: Option<u16>,
    #[serde(default = "default_log_level")]
    pub log_level: LogLevel,
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
    pub web_server_port: Option<u16>,
    pub bind_address: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn to_level_filter(&self) -> LevelFilter {
        match self {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

fn default_log_level() -> LogLevel {
    LogLevel::Info
}

fn default_create_database() -> bool {
    false
}

fn default_log_dir() -> String {
    String::from("~/log/solar_meter")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MeterType {
    Sdm72d,
    Mock,
}

#[derive(Debug, Deserialize)]
pub struct MeterConfig {
    pub name: String,
    pub port: String,
    pub baud_rate: u32,
    pub timeout: u32,
    pub polling_rate: u32,
    pub modbus_address: u8,
    #[serde(flatten)]
    pub meter_type: MeterType,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub global: GlobalConfig,
    pub meters: HashMap<String, MeterConfig>,
}

impl AppConfig {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_locations = vec![
            String::from("./config.toml"),
            String::from("../config.toml"),
            String::from("/etc/solarmeter/config.toml"),
            dirs::home_dir()
                .map(|p| p.join("solarmeter/config.toml").to_string_lossy().into_owned())
                .unwrap_or_else(|| String::from("~/solarmeter/config.toml")),
        ];

        let mut last_error = None;
        
        for path in &config_locations {
            match Self::from_file(path) {
                Ok(config) => return Ok(config),
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| "No config file found".into()))
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&contents)?;
        
        // Convert relative database path to absolute if necessary
        if !Path::new(&config.global.database_url).is_absolute() {
            let config_dir = path.parent().unwrap_or(Path::new("."));
            let absolute_db_path = config_dir.join(&config.global.database_url);
            config.global.database_url = absolute_db_path.to_string_lossy().into_owned();
        }
        
        Ok(config)
    }
}