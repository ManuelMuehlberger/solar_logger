// src/config.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    pub port: String,
    pub baud_rate: u32,
    pub timeout: u32,
    pub polling_rate: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeterConfig {
    pub name: String,
    pub modbus_address: u8,
    #[serde(default)]
    pub input_registers: HashMap<String, u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub global: GlobalConfig,
    pub meters: HashMap<String, MeterConfig>,
}

impl AppConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_str)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_parsing() {
        let config = AppConfig::from_file("/Users/manu/Documents/Privat/Privat/solarmeter/backend/src/config.toml").expect("Failed to parse config");
        println!("{:?}", config);
    }
}