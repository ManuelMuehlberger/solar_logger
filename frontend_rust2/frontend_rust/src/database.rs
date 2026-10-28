use rusqlite::{Connection, params};
use half::f16;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::collections::HashMap;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub meter_name: String,
    pub timestamp: DateTime<Utc>,
    pub total_power: f32,
    pub import_power: f32,
    pub export_power: f32,
    pub total_kwh: f32,
}

pub struct DatabaseSync {
    conn: Mutex<Connection>,
    database_url: String,
    meter_cache: Mutex<HashMap<String, u8>>,
}

impl DatabaseSync {
    pub fn get_connection(&self) -> Result<std::sync::MutexGuard<Connection>, Box<dyn std::error::Error>> {
        Ok(self.conn.lock().map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?)
    }

    pub fn get_database_path(&self) -> String {
        self.database_url.clone()
    }
    
    pub fn f32_to_f16(value: f32) -> i16 {
        f16::from_f32(value).to_bits() as i16
    }

    pub fn f16_to_f32(value: i16) -> f32 {
        f16::from_bits(value as u16).to_f32()
    }

    pub fn new(database_url: &str, create_database: bool) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(parent) = Path::new(database_url).parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(database_url)?;
        
        if create_database {
            // Create meter names lookup table with u8 primary key
            conn.execute(
                "CREATE TABLE IF NOT EXISTS meter_names (
                    meter_id INTEGER PRIMARY KEY CHECK (meter_id >= 0 AND meter_id <= 255),
                    name TEXT NOT NULL UNIQUE
                )",
                [],
            )?;
            
            // Create your other tables here...
        }
        
        Ok(Self {
            conn: Mutex::new(conn),
            database_url: database_url.to_string(),
            meter_cache: Mutex::new(HashMap::new()),
        })
    }
    
    // Add a method to fetch data that we'll display in the UI
    pub fn get_meter_data(&self) -> Result<Vec<Model>, Box<dyn std::error::Error>> {
        // This is a placeholder - implement the actual query based on your database schema
        let mut data = Vec::new();
        
        // Example implementation would be:
        // let conn = self.get_connection()?;
        // let mut stmt = conn.prepare("SELECT meter_name, timestamp, total_power, import_power, export_power, total_kwh FROM your_table")?;
        // let rows = stmt.query_map([], |row| {
        //    // Map row data to Model struct
        // })?;
        
        // For now, returning empty vector
        Ok(data)
    }
}