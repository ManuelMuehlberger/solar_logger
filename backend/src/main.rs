mod config;
mod database_sync;
mod meters;

use std::thread;
use std::time::Duration;
use crate::database_sync::DatabaseSync;
use crate::config::AppConfig;
use crate::meters::create_meter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::from_file("/Users/manu/Documents/Privat/Privat/solarmeter/backend/src/config.toml")?;
    let db_sync = DatabaseSync::new(&config.global.database_url)?;
    
    let meters: Vec<_> = config.meters.iter().map(|(_, meter_config)| {
        create_meter(
            meter_config.name.clone(),
            meter_config.meter_type.clone(),
            config.global.baud_rate,
            meter_config.modbus_address,
            config.global.timeout,
        )
    }).collect();

    println!("Starting meter reading loop");
    loop {
        for meter in &meters {
            match meter.get_value() {
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
        }
        thread::sleep(Duration::from_secs(config.global.polling_rate));
    }
}