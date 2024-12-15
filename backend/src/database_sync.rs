use rusqlite::{Connection, params};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: i32,
    pub meter_name: String,
    pub timestamp: DateTime<Utc>,
    pub total_power: f64,
    pub import_power: f64,
    pub export_power: f64,
    pub total_kwh: f64,
}

pub struct DatabaseSync {
    conn: Connection,
}

impl DatabaseSync {
    pub fn new(database_url: &str, create_database: bool) -> Result<Self, Box<dyn std::error::Error>> {
        // Check if database file exists
        if !Path::new(database_url).exists() {
            if !create_database {
                return Err("Database does not exist and create_database is set to false"
                    .into());
            }
            println!("Database not found! Creating one...");
            // Create directory structure if create_database is true
            if let Some(parent) = Path::new(database_url).parent() {
                fs::create_dir_all(parent)?;
            }
        }

        let conn = Connection::open(database_url)?;
        
        if create_database {
            conn.execute(
                "CREATE TABLE IF NOT EXISTS meter_readings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    meter_name TEXT NOT NULL,
                    timestamp TEXT NOT NULL,
                    total_power REAL NOT NULL,
                    import_power REAL NOT NULL,
                    export_power REAL NOT NULL,
                    total_kwh REAL NOT NULL
                )",
                [],
            )?;
        }

        Ok(Self { conn })
    }

    
    pub fn insert_meter_reading(&self, reading: &Model) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO meter_readings 
            (meter_name, timestamp, total_power, import_power, export_power, total_kwh)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                reading.meter_name,
                reading.timestamp.to_rfc3339(),
                reading.total_power,
                reading.import_power,
                reading.export_power,
                reading.total_kwh,
            ],
        )?;
        Ok(())
    }

    pub fn get_meter_readings(&self, meter_name: &str) -> Result<Vec<Model>, Box<dyn std::error::Error>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, meter_name, timestamp, total_power, import_power, export_power, total_kwh 
             FROM meter_readings 
             WHERE meter_name = ?1 
             ORDER BY timestamp DESC"
        )?;

        let readings = stmt.query_map([meter_name], |row| {
            Ok(Model {
                id: row.get(0)?,
                meter_name: row.get(1)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .unwrap()
                    .with_timezone(&Utc),
                total_power: row.get(3)?,
                import_power: row.get(4)?,
                export_power: row.get(5)?,
                total_kwh: row.get(6)?,
            })
        })?;

        Ok(readings.collect::<Result<Vec<_>, _>>()?)
    }
}
