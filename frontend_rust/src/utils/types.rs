use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub database_size_bytes: u64,
    pub database_path: String,
    pub meters_count: usize,
    pub last_write: Option<i64>,
    pub total_records: i64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeterStatus {
    pub meter_name: String,
    pub last_reading_timestamp: Option<i64>,
    pub last_power_reading: f32,
    pub total_readings: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherInfo {
    pub temperature: f32,
    pub humidity: f32,
    pub description: String,
    pub sunrise: String,
    pub sunset: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub total_import: f32,
    pub total_export: f32,
    pub peak_power: f32,
    pub average_power: f32,
}