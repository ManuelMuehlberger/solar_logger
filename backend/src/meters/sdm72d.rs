// In meters/sdm72d.rs

use tokio_modbus::prelude::*;
use tokio_serial::SerialStream;
use std::time::Duration;
use async_trait::async_trait;
use super::{MeterReader, SharedSerial};
use crate::database_sync::Model;
use chrono::Utc;
use anyhow::{Context, Result, Error};
use std::sync::Arc;

#[allow(dead_code)]
mod registers {
    pub const TOTAL_POWER: u16 = 0x34;      // Total system power (W)
    pub const IMPORT_POWER: u16 = 0x500;    // Import power (W)
    pub const EXPORT_POWER: u16 = 0x502;    // Export power (W)
    pub const TOTAL_ENERGY: u16 = 0x156;    // Total energy (kWh)
}

pub struct SDM72DMeter {
    name: String,
    shared_serial: Arc<SharedSerial>,
    modbus_address: u8,
    polling_rate: u32,
}

impl SDM72DMeter {
    pub fn new(
        name: String,
        shared_serial: Arc<SharedSerial>,
        modbus_address: u8,
        polling_rate: u32,
    ) -> Self {
        Self {
            name,
            shared_serial,
            modbus_address,
            polling_rate,
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

    async fn ensure_connected(&self) -> Result<(), Error> {
        let mut ctx_guard = self.shared_serial.ctx.lock().await;
        if ctx_guard.is_none() {
            let builder = tokio_serial::new(&self.shared_serial.port, self.shared_serial.baud_rate)
                .data_bits(tokio_serial::DataBits::Eight)
                .stop_bits(tokio_serial::StopBits::One)
                .parity(tokio_serial::Parity::None)
                .timeout(Duration::from_secs(self.shared_serial.timeout.into()));

            let serial = SerialStream::open(&builder)
                .context(format!("Failed to open serial port {}", self.shared_serial.port))?;

            *ctx_guard = Some(rtu::attach_slave(serial, Slave(self.modbus_address)));
        }
        Ok(())
    }

    async fn read_float_register(&self, address: u16) -> Result<f32, Error> {
        self.ensure_connected().await?;
        let mut ctx_guard = self.shared_serial.ctx.lock().await;
    
        if let Some(ctx) = &mut *ctx_guard {
            ctx.set_slave(Slave(self.modbus_address));
            
            let registers = ctx
                .read_input_registers(address, 2)
                .await
                .context(format!("Failed to read registers at address {:#04x}", address))?; // Extract the Vec<u16> with `?`
            
            let registers_vec = registers?;
            if registers_vec.len() < 2 {
                anyhow::bail!("Expected 2 registers, got {}", registers_vec.len());
            }
    
            Ok(Self::registers_to_f32(&registers_vec[0..2]))
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
        let total_power = self.read_float_register(registers::TOTAL_POWER)
            .await
            .context("Failed to read total power")?;

        let import_power = self.read_float_register(registers::IMPORT_POWER)
            .await
            .context("Failed to read import power")?;

        let export_power = self.read_float_register(registers::EXPORT_POWER)
            .await
            .context("Failed to read export power")?;

        let total_kwh = self.read_float_register(registers::TOTAL_ENERGY)
            .await
            .context("Failed to read total energy")?;

        Ok(Model {
            meter_name: self.name.clone(),
            timestamp: Utc::now(),
            total_power,
            import_power,
            export_power,
            total_kwh,
        })
    }

    fn get_timeout(&self) -> Duration {
        Duration::from_secs(self.shared_serial.timeout.into())
    }
}