use serde::{Deserialize, Serialize};
use crate::config::MeterType;  // Update this import
use crate::database_sync::Model;

mod sdm72d;
mod mock_meter;
pub use sdm72d::SDM72DMeter;
pub use mock_meter::MockMeter;

pub trait MeterReader: Send + Sync {
    fn get_value(&self) -> Result<Model, Box<dyn std::error::Error>>;
}

pub fn create_meter(
    name: String,
    meter_type: MeterType,
    baud_rate: u32,
    modbus_address: u8,
    timeout: u32,
) -> Box<dyn MeterReader> {
    match meter_type {
        MeterType::Sdm72d => {
            Box::new(SDM72DMeter::new(
                name,
                baud_rate,
                modbus_address,
                timeout,
            ))
        }
        MeterType::Mock { min_power, max_power, power_variation } => {
            Box::new(MockMeter::new(
                name,
                min_power,
                max_power,
                power_variation,
            ))
        }
    }
}