use warp::{Filter, Reply};
use serde::Serialize;
use std::sync::Arc;
use log::{error, info};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::Path;
use std::convert::Infallible;
use tokio::sync::oneshot;
use std::sync::Mutex;

use crate::database_sync::DatabaseSync;

#[derive(Serialize)]
struct SystemStatus {
    database_size_bytes: u64,
    database_path: String,
    meters_count: usize,
    last_write: Option<i64>,  // Unix timestamp as i64
    total_records: i64,
    uptime_seconds: u64,
}

#[derive(Serialize)]
struct MeterStatus {
    meter_name: String,
    last_reading_timestamp: Option<i64>,  // Unix timestamp as i64
    last_power_reading: f32,
    total_readings: i64,
}

#[derive(Clone)]
pub struct WebServer {
    db: Arc<DatabaseSync>,
    start_time: DateTime<Utc>,
    bind_address: String,
    shutdown_sender: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl WebServer {
    pub fn new(db: Arc<DatabaseSync>, bind_address: Option<String>, shutdown_sender: oneshot::Sender<()>) -> Self {
        Self {
            db,
            start_time: Utc::now(),
            bind_address: bind_address.unwrap_or_else(|| "127.0.0.1".to_string()),
            shutdown_sender: Arc::new(Mutex::new(Some(shutdown_sender))),
        }
    }

    fn get_database_size(&self, path: &str) -> Result<u64, std::io::Error> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.len())
    }

    fn get_last_write(&self, conn: &Connection) -> Option<i64> {
        // Get the most recent timestamp directly as i64
        match conn.query_row(
            "SELECT MAX(timestamp) FROM meter_readings",
            [],
            |row| row.get::<_, Option<i64>>(0)
        ) {
            Ok(timestamp) => timestamp,
            Err(e) => {
                error!("Failed to get last write timestamp: {}", e);
                None
            }
        }
    }

    async fn handle_kill(&self) -> Result<impl Reply, Infallible> {
        info!("Kill command received, initiating shutdown...");
        
        let mut sender_guard = self.shutdown_sender.lock().unwrap();
        if let Some(sender) = sender_guard.take() {
            let _ = sender.send(());
            Ok(warp::reply::html(
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Solar Meter Shutdown</title>
                    <style>
                        body { font-family: Arial, sans-serif; margin: 40px; }
                        .message { padding: 20px; background: #f8d7da; border: 1px solid #f5c6cb; border-radius: 4px; }
                    </style>
                </head>
                <body>
                    <div class="message">
                        <h2>Shutdown Initiated</h2>
                        <p>The solar meter monitoring system is shutting down...</p>
                    </div>
                </body>
                </html>
                "#
            ))
        } else {
            Ok(warp::reply::html(
                r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>Solar Meter Shutdown Failed</title>
                    <style>
                        body { font-family: Arial, sans-serif; margin: 40px; }
                        .message { padding: 20px; background: #f8d7da; border: 1px solid #f5c6cb; border-radius: 4px; }
                    </style>
                </head>
                <body>
                    <div class="message">
                        <h2>Shutdown Failed</h2>
                        <p>Shutdown has already been initiated or the shutdown mechanism is not available.</p>
                    </div>
                </body>
                </html>
                "#
            ))
        }
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
                m.name,
                MAX(r.timestamp) as last_reading,
                (SELECT total_power 
                 FROM meter_readings mr2 
                 WHERE mr2.meter_id = r.meter_id
                 ORDER BY timestamp DESC 
                 LIMIT 1) as last_power,
                COUNT(*) as total_readings
             FROM meter_readings r
             JOIN meter_names m ON r.meter_id = m.meter_id
             GROUP BY m.name"
        ) {
            Ok(mut stmt) => {
                match stmt.query_map([], |row| {
                    let last_timestamp: Option<i64> = row.get(1)?;
                    Ok(MeterStatus {
                        meter_name: row.get(0)?,
                        last_reading_timestamp: last_timestamp,  // This will be the Unix timestamp
                        last_power_reading: DatabaseSync::f16_to_f32(row.get::<_, i16>(2)?),
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
            "SELECT COUNT(DISTINCT meter_id) FROM meter_readings",
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

        let kill_route = warp::path("kill")
            .and(warp::get())
            .and(with_server(self.clone()))
            .and_then(|server: WebServer| async move {
                server.handle_kill().await
            });

        let routes = status_route
            .or(meters_route)
            .or(kill_route);

        let addr: std::net::IpAddr = self.bind_address.parse()
            .expect("Invalid bind address");

        println!("Starting web server on {}:{}", addr, port);
        
        warp::serve(routes)
            .run((addr, port))
            .await;
    }
}

// Helper function to pass the WebServer instance to filters
fn with_server(server: WebServer) -> impl Filter<Extract = (WebServer,), Error = Infallible> + Clone {
    warp::any().map(move || server.clone())
}