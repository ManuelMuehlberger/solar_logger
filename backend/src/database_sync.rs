use rusqlite::{Connection, params};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use std::sync::Mutex;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    pub id: i32,
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
}

impl DatabaseSync {

    pub fn get_connection(&self) -> Result<std::sync::MutexGuard<Connection>, Box<dyn std::error::Error>> {
        Ok(self.conn.lock().map_err(|e| Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?)
    }

    pub fn get_database_path(&self) -> String {
        self.database_url.clone()
    }

    pub fn new(database_url: &str, create_database: bool) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(parent) = Path::new(database_url).parent() {
            fs::create_dir_all(parent)?;
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
                    total_kwh REAL NOT NULL,
                    UNIQUE(meter_name, timestamp)
                )",
                [],
            )?;

            conn.execute(
                "CREATE INDEX IF NOT EXISTS idx_meter_timestamp 
                 ON meter_readings (meter_name, timestamp)",
                [],
            )?;
        }

        Ok(Self { 
            conn: Mutex::new(conn),
            database_url: database_url.to_string(),
        })
    }

    pub fn insert_meter_reading(&self, reading: &Model) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO meter_readings 
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

    pub fn get_meter_readings(
        &self,
        meter_name: &str,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<Model>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().unwrap();
        let mut query = String::from(
            "SELECT * FROM meter_readings WHERE meter_name = ?"
        );
        
        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(meter_name)];
        
        if let Some(start) = start_time {
            query.push_str(" AND timestamp >= ?");
            params.push(Box::new(start.to_rfc3339()));
        }
        
        if let Some(end) = end_time {
            query.push_str(" AND timestamp <= ?");
            params.push(Box::new(end.to_rfc3339()));
        }
        
        query.push_str(" ORDER BY timestamp DESC");
        
        let mut stmt = conn.prepare(&query)?;
        let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        
        let readings = stmt.query_map(param_refs.as_slice(), |row| {
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