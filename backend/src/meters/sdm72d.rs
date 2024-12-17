// src/meters/sdm72d.rs
use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;
use std::time::Duration;
use async_trait::async_trait;
use super::MeterReader;
use crate::database_sync::Model;
use chrono::Utc;
use anyhow::{Context, Result, Error};

#[allow(dead_code)]
mod registers {
    pub const TOTAL_POWER: u16 = 0x34;      // Total system power (W)
    pub const IMPORT_POWER: u16 = 0x500;    // Import power (W)
    pub const EXPORT_POWER: u16 = 0x502;    // Export power (W)
    pub const TOTAL_ENERGY: u16 = 0x156;    // Total energy (kWh)
}
 
pub struct SDM72DMeter {
    name: String,
    port: String,
    baud_rate: u32,
    modbus_address: u8,
    timeout: Duration,
    polling_rate: u32,
    ctx: Option<tokio_modbus::client::Context>,
}

impl SDM72DMeter {
    pub fn new(name: String, port: String, baud_rate: u32, modbus_address: u8, timeout: u32, polling_rate: u32) -> Self {
        Self {
            name,
            port,
            baud_rate,
            modbus_address,
            polling_rate,
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

    async fn ensure_connected(&mut self) -> Result<(), Error> {
        if self.ctx.is_none() {
            let builder = tokio_serial::new(&self.port, self.baud_rate)
                .data_bits(tokio_serial::DataBits::Eight)
                .stop_bits(tokio_serial::StopBits::One)
                .parity(tokio_serial::Parity::None)
                .timeout(self.timeout);

            let serial = SerialStream::open(&builder)
                .context(format!("Failed to open serial port {}", self.port))?;

            self.ctx = Some(rtu::attach_slave(serial, Slave(self.modbus_address)));
        }
        Ok(())
    }

    async fn read_float_register(&mut self, address: u16) -> Result<f32, Error> {
        self.ensure_connected().await?;
        
        if let Some(ctx) = &mut self.ctx {
            let registers = ctx.read_input_registers(address, 2)
                .await
                .context(format!("Failed to read registers at address {:#04x}", address))?;

            if registers.clone()?.len() < 2 {
                anyhow::bail!("Expected 2 registers, got {}", registers?.len());
            }

            Ok(Self::registers_to_f32(&registers?[0..2]))
        } else {
            anyhow::bail!("Modbus context not initialized")
        }
    }
}


#[async_trait]
impl MeterReader for SDM72DMeter {
    fn get_polling_rate(&self) -> u32 {
        self.polling_rate
    }

    async fn get_value(&mut self) -> Result<Model, Error> {
        let total_power = self.read_float_register(0x34)
            .await
            .context("Failed to read total power")?;

        let import_power = self.read_float_register(0x500)
            .await
            .context("Failed to read import power")?;

        let export_power = self.read_float_register(0x502)
            .await
            .context("Failed to read export power")?;

        let total_kwh = self.read_float_register(0x156)
            .await
            .context("Failed to read total energy")?;

        Ok(Model {
            id: 0,
            meter_name: self.name.clone(),
            timestamp: Utc::now(),
            total_power,
            import_power,
            export_power,
            total_kwh,
        })
    }

    fn get_timeout(&self) -> Duration {
        self.timeout
    }
}