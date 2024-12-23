use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub database_size_bytes: u64,
    pub database_path: String,
    pub meters_count: usize,
    pub last_write: Option<i64>,
    pub total_records: i64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeterStatus {
    pub meter_name: String,
    #[serde(default)]
    pub last_reading_timestamp: Option<i64>,
    #[serde(deserialize_with = "deserialize_power")]
    pub last_power_reading: f32,
    pub total_readings: i64,
}

// Special deserialization for power values that might come in different formats
pub fn deserialize_power<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    
    match value {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Ok(f as f32)
            } else {
                Ok(0.0)
            }
        },
        Value::String(s) => {
            s.parse::<f32>().map_err(serde::de::Error::custom)
        },
        _ => Ok(0.0),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherInfo {
    pub temperature: f32,
    pub humidity: i32,
    pub description: String,
    pub sunrise: String,
    pub sunset: String,
}