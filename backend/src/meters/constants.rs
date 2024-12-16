pub mod registers {
    // Input Registers
    pub const TOTAL_POWER: u16 = 0x0034;     // Total system power (W)
    pub const IMPORT_ENERGY: u16 = 0x0048;   // Import Wh since last reset (kWh)
    pub const EXPORT_ENERGY: u16 = 0x004A;   // Export Wh since last reset (kWh)
    pub const TOTAL_KWH: u16 = 0x0156;       // Total kWh
    pub const IMPORT_POWER: u16 = 0x0500;    // Import power (W)
    pub const EXPORT_POWER: u16 = 0x0502;    // Export power (W)

    // Holding Registers (Configuration)
    pub const MODBUS_ADDRESS: u16 = 0x0014;  // Device Modbus address
    pub const BAUD_RATE: u16 = 0x001C;       // Communication baud rate
}
