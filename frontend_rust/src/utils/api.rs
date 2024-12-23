use axum::{
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use super::types::{SystemStatus, MeterStatus, WeatherInfo};
use tracing::{info, error};

const BACKEND_URL: &str = "http://localhost:8081";

pub async fn get_status() -> impl IntoResponse {
    match reqwest::get(&format!("{}/status", BACKEND_URL)).await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<SystemStatus>().await {
                    Ok(status) => {
                        info!("Received status: {:?}", status);
                        (StatusCode::OK, Json(status)).into_response()
                    },
                    Err(e) => {
                        error!("Failed to parse status response: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }
            } else {
                error!("Backend returned error status: {}", response.status());
                StatusCode::BAD_GATEWAY.into_response()
            }
        }
        Err(e) => {
            error!("Failed to connect to backend: {}", e);
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

pub async fn get_meters() -> impl IntoResponse {
    match reqwest::get(&format!("{}/meters", BACKEND_URL)).await {
        Ok(response) => {
            if response.status().is_success() {
                // First log the raw response
                let text = response.text().await.unwrap_or_default();
                info!("Raw meter response: {}", text);
                
                // Parse the text back into JSON
                match serde_json::from_str::<Vec<MeterStatus>>(&text) {
                    Ok(meters) => {
                        info!("Parsed meters: {:?}", meters);
                        (StatusCode::OK, Json(meters)).into_response()
                    },
                    Err(e) => {
                        error!("Failed to parse meters response: {}", e);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }
            } else {
                error!("Backend returned error status: {}", response.status());
                StatusCode::BAD_GATEWAY.into_response()
            }
        }
        Err(e) => {
            error!("Failed to connect to backend: {}", e);
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

pub async fn get_weather() -> impl IntoResponse {
    let client = reqwest::Client::new();
    
    match client.get(&format!("{}/weather", BACKEND_URL))
        .send()
        .await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<WeatherInfo>().await {
                        Ok(weather) => {
                            info!("Received weather: {:?}", weather);
                            (StatusCode::OK, Json(weather)).into_response()
                        },
                        Err(e) => {
                            error!("Failed to parse weather response: {}", e);
                            StatusCode::INTERNAL_SERVER_ERROR.into_response()
                        }
                    }
                } else {
                    error!("Backend returned error status: {}", response.status());
                    StatusCode::BAD_GATEWAY.into_response()
                }
            }
            Err(e) => {
                error!("Failed to connect to backend: {}", e);
                StatusCode::SERVICE_UNAVAILABLE.into_response()
            }
        }
}