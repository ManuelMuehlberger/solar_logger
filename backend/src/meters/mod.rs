// In meters/mod.rs
use async_trait::async_trait;
use anyhow::Error;
use std::time::Duration;
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio_modbus::client::Context;
use std::collections::HashMap;
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

pub struct SharedSerial {
    ctx: Mutex<Option<Context>>,
    port: String,
    baud_rate: u32,
    timeout: u32,
}

impl SharedSerial {
    pub fn new(port: String, baud_rate: u32, timeout: u32) -> Arc<Self> {
        Arc::new(Self {
            ctx: Mutex::new(None),
            port,
            baud_rate,
            timeout,
        })
    }
}

// Global storage for shared serial connections
lazy_static::lazy_static! {
    static ref SHARED_SERIALS: Mutex<HashMap<String, Arc<SharedSerial>>> = Mutex::new(HashMap::new());
}

pub async fn get_or_create_shared_serial(port: String, baud_rate: u32, timeout: u32) -> Arc<SharedSerial> {
    let mut serials = SHARED_SERIALS.lock().await;
    if let Some(serial) = serials.get(&port) {
        Arc::clone(serial)
    } else {
        let serial = SharedSerial::new(port.clone(), baud_rate, timeout);
        serials.insert(port, Arc::clone(&serial));
        serial
    }
}

pub async fn create_meter(
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
            let shared_serial = get_or_create_shared_serial(port, baud_rate, timeout).await;
            Box::new(SDM72DMeter::new(
                name,
                shared_serial,
                modbus_address,
                polling_rate
            ))
        }
        crate::config::MeterType::Mock => {
            Box::new(MockMeter::new(name))
        }
    }
}