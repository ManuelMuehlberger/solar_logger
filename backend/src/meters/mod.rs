use async_trait::async_trait;
use crate::database_sync::Model;
use std::time::Duration;

mod constants;
mod mock_meter;
mod sdm72d;

pub use mock_meter::MockMeter;
pub use sdm72d::SDM72DMeter;

#[async_trait]
pub trait MeterReader {
    async fn get_value(&mut self) -> Result<Model, Box<dyn std::error::Error>>;

    fn get_timeout(&mut self) -> Duration;
}

pub fn create_meter(
    name: String,
    meter_type: crate::config::MeterType,
    port: String,
    baud_rate: u32,
    polling_rate: u32,
    modbus_address: u8,
    timeout: u32,
) -> Box<dyn MeterReader> {
    match meter_type {
        crate::config::MeterType::Sdm72d => {
            Box::new(SDM72DMeter::new(name, port, baud_rate, polling_rate, modbus_address, timeout))
        }
        crate::config::MeterType::Mock {
            min_power,
            max_power,
            power_variation,
        } => Box::new(MockMeter::new(name, min_power, max_power, power_variation)),
    }
}