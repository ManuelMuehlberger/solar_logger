use async_trait::async_trait;
use crate::database_sync::Model;

mod constants;
mod mock_meter;
mod sdm72d;

pub use mock_meter::MockMeter;
pub use sdm72d::SDM72DMeter;

#[async_trait]
pub trait MeterReader {
    async fn get_value(&mut self) -> Result<Model, Box<dyn std::error::Error>>;
}

pub fn create_meter(
    name: String,
    meter_type: crate::config::MeterType,
    baud_rate: u32,
    modbus_address: u8,
    timeout: u32,
) -> Box<dyn MeterReader> {
    match meter_type {
        crate::config::MeterType::Sdm72d => {
            Box::new(SDM72DMeter::new(name, baud_rate, modbus_address, timeout))
        }
        crate::config::MeterType::Mock {
            min_power,
            max_power,
            power_variation,
        } => Box::new(MockMeter::new(name, min_power, max_power, power_variation)),
    }
}