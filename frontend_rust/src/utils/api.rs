use crate::utils::types::{SystemStatus, MeterStatus};
use reqwest::Client;
use std::sync::OnceLock;

static CLIENT: OnceLock<Client> = OnceLock::new();

const API_BASE_URL: &str = "http://localhost:8081";

fn get_client() -> &'static Client {
    CLIENT.get_or_init(Client::new)
}

pub async fn get_system_status() -> Result<SystemStatus, reqwest::Error> {
    get_client()
        .get(&format!("{}/status", API_BASE_URL))
        .send()
        .await?
        .json()
        .await
}

pub async fn get_meter_status() -> Result<Vec<MeterStatus>, reqwest::Error> {
    get_client()
        .get(&format!("{}/meters", API_BASE_URL))
        .send()
        .await?
        .json()
        .await
}

// We'll add more API functions as needed for charts and other data
pub async fn get_current_power_usage() -> Result<f32, reqwest::Error> {
    let meters = get_meter_status().await?;
    Ok(meters.iter().map(|m| m.last_power_reading.abs()).sum())
}