// src/main.rs
mod config;
use config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = AppConfig::from_file("/Users/manu/Documents/Privat/Privat/solarmeter/backend/src/config.toml")?;
    
    // Print global configuration
    println!("Global Config: {:?}", config.global);
    
    // Print meter configurations
    for (name, meter_config) in &config.meters {
        println!("Meter {}: {:?}", name, meter_config);
    }

    Ok(())
}