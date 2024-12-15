pub trait ModbusClient {
    fn read_input_registers(
        &mut self,
        address: u16,
        quantity: u16,
    ) -> Result<Vec<u16>, Box<dyn std::error::Error>>;
}

pub struct RealModbusClient {
    client: Client,
}

impl RealModbusClient {
    pub fn new(port: &str, baud_rate: u32, timeout: u32, slave: u8) -> Result<Self, Box<dyn std::error::Error>> {
        let builder = tokio_serial::new(port, baud_rate)
            .timeout(std::time::Duration::from_secs(timeout.into()));
        let transport = SerialClient::new_rtu(builder, slave)?;
        Ok(Self {
            client: transport.attach(),
        })
    }
}

impl ModbusClient for RealModbusClient {
    fn read_input_registers(
        &mut self,
        address: u16,
        quantity: u16,
    ) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
        Ok(vec![0; quantity as usize])
    }
}

pub struct MockModbusClient;

impl ModbusClient for MockModbusClient {
    fn read_input_registers(
        &mut self,
        _address: u16,
        quantity: u16,
    ) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
        Ok(vec![0; quantity as usize])
    }
}