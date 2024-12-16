mod config;
mod database_sync;
mod meters;

use tokio::time::{sleep, Duration};
use crate::database_sync::DatabaseSync;
use crate::config::AppConfig;
use crate::meters::create_meter;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AppConfig::from_file("src/config.toml")?;
    println!("Configuration loaded successfully");

    // Initialize database
    let db_sync = Arc::new(DatabaseSync::new(
        &config.global.database_url,
        config.global.create_database
    )?);
    println!("Database initialized");

    // Create meters
    let mut meters = Vec::new();
    for (_, meter_config) in &config.meters {
        let meter = create_meter(
            meter_config.name.clone(),
            meter_config.meter_type.clone(),
            meter_config.port.clone(),
            meter_config.baud_rate,
            meter_config.polling_rate,
            meter_config.modbus_address,
            meter_config.timeout,
        );
        meters.push(meter);
    }
    println!("Created {} meters", meters.len());

    println!("Starting meter reading loop");
    loop {
        for meter in &mut meters {
            match meter.get_value().await {
                Ok(reading) => {
                    println!("Reading from {}: {:?}", reading.meter_name, reading);
                    if let Err(e) = db_sync.insert_meter_reading(&reading) {
                        eprintln!("Error inserting reading: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("Error reading meter: {}", e);
                }
            }
            sleep(meter.get_timeout()).await;
        }
        
        
    }
}