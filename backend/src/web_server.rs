use warp::{Filter, Reply};
use serde::Serialize;
use std::sync::Arc;
use log::error;
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::Path;
use std::convert::Infallible;

use crate::database_sync::DatabaseSync;

#[derive(Serialize)]
struct SystemStatus {
    database_size_bytes: u64,
    database_path: String,
    meters_count: usize,
    last_write: Option<DateTime<Utc>>,
    total_records: i64,
    uptime_seconds: u64,
}

#[derive(Serialize)]
struct MeterStatus {
    meter_name: String,
    last_reading_timestamp: Option<DateTime<Utc>>,
    last_power_reading: Option<f32>,
    total_readings: i64,
}

#[derive(Clone)]
pub struct WebServer {
    db: Arc<DatabaseSync>,
    start_time: DateTime<Utc>,
}

impl WebServer {
    pub fn new(db: Arc<DatabaseSync>) -> Self {
        Self {
            db,
            start_time: Utc::now(),
        }
    }

    fn get_database_size(&self, path: &str) -> Result<u64, std::io::Error> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.len())
    }

    fn get_last_write(&self, conn: &Connection) -> Option<DateTime<Utc>> {
        conn.query_row(
            "SELECT MAX(timestamp) FROM meter_readings",
            [],
            |row| {
                let timestamp: Option<String> = row.get(0)?;
                Ok(timestamp.and_then(|ts| DateTime::parse_from_rfc3339(&ts).ok().map(|dt| dt.with_timezone(&Utc))))
            }
        ).ok().flatten()
    }

    async fn handle_status(&self) -> Result<impl Reply, Infallible> {
        let conn = match self.db.get_connection() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get database connection: {}", e);
                return Ok(warp::reply::json(&SystemStatus {
                    database_size_bytes: 0,
                    database_path: String::from("unknown"),
                    meters_count: 0,
                    last_write: None,
                    total_records: 0,
                    uptime_seconds: 0,
                }));
            }
        };

        let db_path = Path::new(&self.db.get_database_path()).to_string_lossy().into_owned();
        
        let status = SystemStatus {
            database_size_bytes: self.get_database_size(&db_path).unwrap_or(0),
            database_path: db_path,
            meters_count: self.get_unique_meters(&conn).unwrap_or(0),
            last_write: self.get_last_write(&conn),
            total_records: self.get_total_records(&conn).unwrap_or(0),
            uptime_seconds: (Utc::now() - self.start_time).num_seconds() as u64,
        };

        Ok(warp::reply::json(&status))
    }

    async fn handle_meters(&self) -> Result<impl Reply, Infallible> {
        let conn = match self.db.get_connection() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get database connection: {}", e);
                return Ok(warp::reply::json(&Vec::<MeterStatus>::new()));
            }
        };

        let meters = match conn.prepare(
            "SELECT 
                meter_name,
                MAX(timestamp) as last_reading,
                (SELECT total_power 
                 FROM meter_readings mr2 
                 WHERE mr2.meter_name = mr1.meter_name 
                 ORDER BY timestamp DESC 
                 LIMIT 1) as last_power,
                COUNT(*) as total_readings
             FROM meter_readings mr1
             GROUP BY meter_name"
        ) {
            Ok(mut stmt) => {
                match stmt.query_map([], |row| {
                    Ok(MeterStatus {
                        meter_name: row.get(0)?,
                        last_reading_timestamp: row.get::<_, Option<String>>(1)?
                            .and_then(|ts| DateTime::parse_from_rfc3339(&ts).ok().map(|dt| dt.with_timezone(&Utc))),
                        last_power_reading: row.get(2)?,
                        total_readings: row.get(3)?,
                    })
                }) {
                    Ok(rows) => rows.filter_map(Result::ok).collect::<Vec<_>>(),
                    Err(e) => {
                        error!("Failed to query meters: {}", e);
                        Vec::new()
                    }
                }
            },
            Err(e) => {
                error!("Failed to prepare statement: {}", e);
                Vec::new()
            }
        };

        Ok(warp::reply::json(&meters))
    }

    fn get_unique_meters(&self, conn: &Connection) -> Result<usize, rusqlite::Error> {
        conn.query_row(
            "SELECT COUNT(DISTINCT meter_name) FROM meter_readings",
            [],
            |row| row.get(0),
        )
    }

    fn get_total_records(&self, conn: &Connection) -> Result<i64, rusqlite::Error> {
        conn.query_row(
            "SELECT COUNT(*) FROM meter_readings",
            [],
            |row| row.get(0),
        )
    }

    pub async fn run(self, port: u16) {
        let status_route = warp::path("status")
            .and(warp::get())
            .and(with_server(self.clone()))
            .and_then(|server: WebServer| async move {
                server.handle_status().await
            });

        let meters_route = warp::path("meters")
            .and(warp::get())
            .and(with_server(self.clone()))
            .and_then(|server: WebServer| async move {
                server.handle_meters().await
            });

        let routes = status_route.or(meters_route);

        warp::serve(routes)
            .run(([127, 0, 0, 1], port))
            .await;
    }
}

// Helper function to pass the WebServer instance to filters
fn with_server(server: WebServer) -> impl Filter<Extract = (WebServer,), Error = Infallible> + Clone {
    warp::any().map(move || server.clone())
}