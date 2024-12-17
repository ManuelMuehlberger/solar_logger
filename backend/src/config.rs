use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub database_url: String,
    #[serde(default = "default_create_database")]
    pub create_database: bool,
    pub health_check_port: Option<u16>,
    pub log_level: Option<String>,
    pub web_server_port: Option<u16>,
    pub bind_address: String,
}

fn default_create_database() -> bool {
    false
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MeterType {
    Sdm72d,
    Mock,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub global: GlobalConfig,
    pub meters: HashMap<String, MeterConfig>,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&contents)?;
        
        // Convert relative paths to absolute if necessary
        if !Path::new(&config.global.database_url).is_absolute() {
            let config_dir = Path::new(path).parent().unwrap_or(Path::new("."));
            let absolute_db_path = config_dir.join(&config.global.database_url);
            config.global.database_url = absolute_db_path.to_string_lossy().into_owned();
        }
        
        Ok(config)
    }
}