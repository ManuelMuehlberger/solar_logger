use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub port: String,
    pub baud_rate: u32,
    pub timeout: u32,
    pub polling_rate: u64,
    pub database_url: String,
}

#[derive(Debug, Deserialize)]
pub struct MeterConfig {
    pub name: String,
    pub modbus_address: u8,
    #[serde(flatten)]
    pub meter_type: MeterType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MeterType {
    Sdm72d,
    Mock {
        min_power: f64,
        max_power: f64,
        power_variation: f64,
    },
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub global: GlobalConfig,
    pub meters: HashMap<String, MeterConfig>,
}

impl AppConfig {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}