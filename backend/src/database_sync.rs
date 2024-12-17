use rusqlite::{Connection, params};
use half::f16;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use std::collections::HashMap;
use chrono::{DateTime, TimeZone, Utc};
use serde::{Serialize, Deserialize};
use std::convert::TryFrom;

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

            // Create optimized readings table
            conn.execute(
                "CREATE TABLE IF NOT EXISTS meter_readings (
                    meter_id INTEGER NOT NULL CHECK (meter_id >= 0 AND meter_id <= 255),
                    timestamp INTEGER NOT NULL,  -- Unix timestamp in seconds
                    total_power SMALLINT NOT NULL,  -- f16 stored as i16
                    import_power SMALLINT NOT NULL, -- f16 stored as i16
                    export_power SMALLINT NOT NULL, -- f16 stored as i16
                    total_kwh REAL NOT NULL,       -- f32 stored as REAL
                    PRIMARY KEY (meter_id, timestamp),
                    FOREIGN KEY (meter_id) REFERENCES meter_names(meter_id)
                )",
                [],
            )?;

            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_meter_timestamp 
                 ON meter_readings (meter_id, timestamp)",
                [],
            )?;
        }

        // Load existing meter names into cache
        let meter_cache = {
            let mut cache = HashMap::new();
            let mut stmt = conn.prepare("SELECT meter_id, name FROM meter_names")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(1)?, row.get::<_, u8>(0)?))
            })?;

            for row in rows {
                let (name, id) = row?;
                cache.insert(name, id);
            }
            cache
        }; // stmt is dropped here

        Ok(Self { 
            conn: Mutex::new(conn),
            database_url: database_url.to_string(),
            meter_cache: Mutex::new(meter_cache),
        })
    }

    fn get_or_create_meter_id(&self, meter_name: &str) -> Result<u8, Box<dyn std::error::Error>> {
        let mut cache = self.meter_cache.lock().unwrap();
        
        if let Some(&id) = cache.get(meter_name) {
            return Ok(id);
        }

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO meter_names (name) 
             SELECT ?1 WHERE NOT EXISTS (
                SELECT 1 FROM meter_names WHERE meter_id = 255
             )",
            params![meter_name],
        )?;

        let meter_id: u8 = conn.query_row(
            "SELECT meter_id FROM meter_names WHERE name = ?1",
            params![meter_name],
            |row| row.get(0),
        ).map_err(|_| "Failed to create meter ID - maximum number of meters (256) reached")?;

        cache.insert(meter_name.to_string(), meter_id);
        Ok(meter_id)
    }

    pub fn insert_meter_reading(&self, reading: &Model) -> Result<(), Box<dyn std::error::Error>> {
        let meter_id = self.get_or_create_meter_id(&reading.meter_name)?;
        let timestamp = i64::try_from(reading.timestamp.timestamp())?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO meter_readings 
            (meter_id, timestamp, total_power, import_power, export_power, total_kwh)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                meter_id,
                timestamp,
                Self::f32_to_f16(reading.total_power),
                Self::f32_to_f16(reading.import_power),
                Self::f32_to_f16(reading.export_power),
                reading.total_kwh,
            ],
        )?;
        Ok(())
    }

    pub fn get_meter_readings(
        &self,
        meter_name: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<Model>, Box<dyn std::error::Error>> {
        let meter_id = self.get_or_create_meter_id(meter_name)?;
        let conn = self.conn.lock().unwrap();

        let mut query = String::from(
            "SELECT m.name, r.timestamp, r.total_power, r.import_power, r.export_power, r.total_kwh 
             FROM meter_readings r 
             JOIN meter_names m ON r.meter_id = m.meter_id 
             WHERE r.meter_id = ?"
        );
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(meter_id)];
        
        if let Some(start) = start_time {
            query.push_str(" AND r.timestamp >= ?");
            params.push(Box::new(start.timestamp()));
        }
        
        if let Some(end) = end_time {
            query.push_str(" AND r.timestamp <= ?");
            params.push(Box::new(end.timestamp()));
        }
        
        query.push_str(" ORDER BY r.timestamp DESC");
        
        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let readings = stmt.query_map(param_refs.as_slice(), |row| {
            Ok(Model {
                meter_name: row.get(0)?,
                timestamp: Utc.timestamp_opt(row.get(1)?, 0).unwrap(),
                total_power: Self::f16_to_f32(row.get(2)?),
                import_power: Self::f16_to_f32(row.get(3)?),
                export_power: Self::f16_to_f32(row.get(4)?),
                total_kwh: row.get(5)?,
            })
        })?;

        Ok(readings.collect::<Result<Vec<_>, _>>()?)
    }
}