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
use log::{debug, error, info};

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
        info!("Initializing SDM72D meter '{}' on port {} at address {}", 
              name, shared_serial.port, modbus_address);
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
        let value = f32::from_be_bytes(bytes);
        debug!("Converted registers [{:04x}, {:04x}] to float: {}", regs[0], regs[1], value);
        value
    }

    async fn ensure_connected(&self) -> Result<(), Error> {
        let mut ctx_guard = self.shared_serial.ctx.lock().await;
        if ctx_guard.is_none() {
            debug!("{}: No existing Modbus context found, creating new connection", self.name);
            
            info!("{}: Opening serial port {} at {} baud", 
                  self.name, self.shared_serial.port, self.shared_serial.baud_rate);

            let builder = tokio_serial::new(&self.shared_serial.port, self.shared_serial.baud_rate)
                .data_bits(tokio_serial::DataBits::Eight)
                .stop_bits(tokio_serial::StopBits::One)
                .parity(tokio_serial::Parity::None)
                .timeout(Duration::from_secs(self.shared_serial.timeout.into()));

            let serial = SerialStream::open(&builder)
                .context(format!("Failed to open serial port {}", self.shared_serial.port))?;

            *ctx_guard = Some(rtu::attach_slave(serial, Slave(self.modbus_address)));
            info!("{}: Successfully initialized Modbus RTU context", self.name);
        }
        Ok(())
    }

    async fn read_float_register(&self, register: u16, description: &str) -> Result<f32, Error> {
        self.ensure_connected().await?;
        let mut ctx_guard = self.shared_serial.ctx.lock().await;
    
        if let Some(ctx) = &mut *ctx_guard {
            debug!("{}: Setting slave address to {} for {}", self.name, self.modbus_address, description);
            ctx.set_slave(Slave(self.modbus_address));
            
            debug!("{}: Reading {} registers at address {:#04x}", self.name, description, register);
            
            let registers = ctx.read_input_registers(register, 2)
                .await
                .context(format!("{}: Failed to read {} registers at {:#04x}", 
                    self.name, description, register))?;
            
            if registers.clone()?.len() < 2 {
                let err = format!("{}: Expected 2 registers for {}, got {}", 
                    self.name, description, registers?.len());
                error!("{}", err);
                return Err(anyhow::Error::msg(err));
            }
    
            let value = Self::registers_to_f32(&registers.clone()?[0..2]);
            info!("{}: {} value: {:.2}", self.name, description, value);
            Ok(value)
        } else {
            let err = format!("{}: Modbus context not initialized for {}", self.name, description);
            error!("{}", err);
            Err(anyhow::Error::msg(err))
        }
    }
}

#[async_trait]
impl MeterReader for SDM72DMeter {
    fn get_polling_rate(&self) -> u32 {
        self.polling_rate
    }

    async fn get_value(&mut self) -> Result<Model, Error> {
        info!("{}: Starting new reading cycle", self.name);

        // Acquire lock before starting communication
        self.shared_serial.acquire_lock(&self.name).await?;

        // Use a closure to ensure we always release the lock
        let result = async {
            let total_power = self.read_float_register(registers::TOTAL_POWER, "Total Power")
                .await?;

            let import_power = self.read_float_register(registers::IMPORT_POWER, "Import Power")
                .await?;

            let export_power = self.read_float_register(registers::EXPORT_POWER, "Export Power")
                .await?;

            let total_kwh = self.read_float_register(registers::TOTAL_ENERGY, "Total Energy")
                .await?;

            info!("{}: Completed reading cycle. Total Power: {:.2}W, Import: {:.2}W, Export: {:.2}W, Total: {:.2}kWh",
                self.name, total_power, import_power, export_power, total_kwh);

            Ok(Model {
                meter_name: self.name.clone(),
                timestamp: Utc::now(),
                total_power,
                import_power,
                export_power,
                total_kwh,
            })
        }.await;

        // Always release the lock
        self.shared_serial.release_lock(&self.name).await;

        result
    }

    fn get_timeout(&self) -> Duration {
        Duration::from_secs(self.shared_serial.timeout.into())
    }
}