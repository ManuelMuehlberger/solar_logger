use async_trait::async_trait;
use anyhow::{Result, Error};
use std::time::Duration;
use crate::database_sync::Model;

mod mock_meter;
mod sdm72d;

pub use mock_meter::MockMeter;
pub use sdm72d::SDM72DMeter;

#[async_trait]
pub trait MeterReader: Send {
    async fn get_value(&mut self) -> Result<Model, Error>;
    fn get_timeout(&self) -> Duration;
    fn get_polling_rate(&self) -> u32;
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
            Box::new(SDM72DMeter::new(
                name,
                port,
                baud_rate,
                modbus_address,
                timeout,
                polling_rate
            ))
        }
        crate::config::MeterType::Mock => {
            Box::new(MockMeter::new(
            name,
        ))
    }
    }
}