use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;
use std::time::Duration;
use async_trait::async_trait;
use super::MeterReader;
use crate::database_sync::Model;
use chrono::Utc;

pub mod registers {
    // Input Registers
    pub const TOTAL_POWER: u16 = 0x0034;     // Total system power (W)
    pub const IMPORT_ENERGY: u16 = 0x0048;   // Import Wh since last reset (kWh)
    pub const EXPORT_ENERGY: u16 = 0x004A;   // Export Wh since last reset (kWh)
    pub const TOTAL_KWH: u16 = 0x0156;       // Total kWh
    pub const IMPORT_POWER: u16 = 0x0500;    // Import power (W)
    pub const EXPORT_POWER: u16 = 0x0502;    // Export power (W)

    // not used
    pub const MODBUS_ADDRESS: u16 = 0x0014;  // Device Modbus address
    pub const BAUD_RATE: u16 = 0x001C;       // Communication baud rate
}

pub struct SDM72DMeter {
    name: String,
    port: String,
    baud_rate: u32,
    modbus_address: u8,
    timeout: Duration,
    ctx: Option<tokio_modbus::client::Context>,
}

impl SDM72DMeter {
    pub fn new(name: String, port: String, baud_rate: u32, modbus_address: u8, timeout: u32) -> Self {
        Self {
            name,
            port,
            baud_rate,
            modbus_address,
            timeout: Duration::from_secs(timeout.into()),
            ctx: None,
        }
    }

    fn registers_to_f32(regs: &[u16]) -> f32 {
        let bytes = [
            (regs[0] >> 8) as u8,
            (regs[0] & 0xFF) as u8,
            (regs[1] >> 8) as u8,
            (regs[1] & 0xFF) as u8,
        ];
        f32::from_be_bytes(bytes)
    }

    async fn ensure_connected(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.ctx.is_none() {
            let builder = tokio_serial::new(&self.port, self.baud_rate)
                .data_bits(tokio_serial::DataBits::Eight)
                .stop_bits(tokio_serial::StopBits::One)
                .parity(tokio_serial::Parity::None)
                .timeout(self.timeout);

            let serial = SerialStream::open(&builder)?;
            self.ctx = Some(rtu::attach_slave(serial, Slave(self.modbus_address)));
        }
        Ok(())
    }

    async fn read_float_register(&mut self, address: u16) -> Result<f32, Box<dyn std::error::Error>> {
        self.ensure_connected().await?;
        
        if let Some(ctx) = &mut self.ctx {
            let registers = ctx.read_input_registers(address, 2).await?;
            if registers.clone()?.len() < 2 {
                return Err(format!("Expected 2 registers, got {}", registers?.len()).into());
            }
            Ok(Self::registers_to_f32(&registers?[0..2]))
        } else {
            Err("Context not initialized".into())
        }
    }
}

#[async_trait]
impl MeterReader for SDM72DMeter {
    async fn get_value(&mut self) -> Result<Model, Box<dyn std::error::Error>> {
        // Read all required registers
        let total_power = self.read_float_register(registers::TOTAL_POWER).await?;
        let import_power = self.read_float_register(registers::IMPORT_POWER).await?;
        let export_power = self.read_float_register(registers::EXPORT_POWER).await?;
        let total_kwh = self.read_float_register(registers::TOTAL_KWH).await?;

        Ok(Model {
            id: 0,
            meter_name: self.name.clone(),
            timestamp: Utc::now(),
            total_power: total_power as f32,
            import_power: import_power as f32,
            export_power: export_power as f32,
            total_kwh: total_kwh as f32,
        })
    }

    fn get_timeout(&mut self) -> Duration {
        self.timeout
    }
}