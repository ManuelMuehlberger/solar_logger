use std::time::Duration;
use super::MeterReader;
use crate::database_sync::Model;
use chrono::Utc;

pub struct SDM72DMeter {
    name: String,
    baud_rate: u32,
    modbus_address: u8,
    timeout: Duration,
}

impl SDM72DMeter {
    pub fn new(name: String, baud_rate: u32, modbus_address: u8, timeout: u32) -> Self {
        Self {
            name,
            baud_rate,
            modbus_address,
            timeout: Duration::from_secs(timeout.into()),
        }
    }

    fn read_float_register(address: u16) -> f64 {
        let bytes = [0x41, 0x48, 0x00, 0x00];
        f32::from_be_bytes(bytes) as f64
    }
}

impl MeterReader for SDM72DMeter {
    fn get_value(&self) -> Result<Model, Box<dyn std::error::Error>> {
        Ok(Model {
            id: 0,
            meter_name: self.name.clone(),
            timestamp: Utc::now(),
            total_power: Self::read_float_register(0x0034),
            import_power: Self::read_float_register(0x0500),
            export_power: Self::read_float_register(0x0502),
            total_kwh: Self::read_float_register(0x0156),
        })
    }
}
