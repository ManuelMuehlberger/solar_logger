use std::sync::Arc;
use tokio::time::{sleep, Duration};
use chrono::{DateTime, Utc, TimeDelta};
use log::{info, error};
use rusqlite::Transaction;

use crate::database_sync::{DatabaseSync, Model};

pub struct RetentionService {
    db: Arc<DatabaseSync>,
}

impl RetentionService {
    pub fn new(db: Arc<DatabaseSync>) -> Self {
        Self { db }
    }

    /// Aggregates data points within a time window into a single data point
    fn aggregate_data_points(readings: &[Model]) -> Option<Model> {
        if readings.is_empty() {
            return None;
        }

        let count = readings.len() as f32;
        let first = &readings[0];

        // Calculate averages for power readings
        let total_power = readings.iter().map(|r| r.total_power).sum::<f32>() / count;
        let import_power = readings.iter().map(|r| r.import_power).sum::<f32>() / count;
        let export_power = readings.iter().map(|r| r.export_power).sum::<f32>() / count;

        // Use the latest total_kwh reading as it's cumulative
        let total_kwh = readings.iter().map(|r| r.total_kwh).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        // Use the timestamp from the middle of the interval
        let middle_idx = readings.len() / 2;
        let timestamp = readings[middle_idx].timestamp;

        Some(Model {
            meter_name: first.meter_name.clone(),
            timestamp,
            total_power,
            import_power,
            export_power,
            total_kwh,
        })
    }

    async fn process_retention(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = Utc::now();

        // Define time windows and their target intervals
        let windows = [
            // (start_age, end_age, interval_minutes)
            (TimeDelta::try_hours(24 * 7).unwrap(), None, 60), // Older than 1 week -> 1 hour
            (TimeDelta::try_hours(24).unwrap(), Some(TimeDelta::try_hours(24 * 7).unwrap()), 20), // 1 day to 1 week -> 20 min
            (TimeDelta::try_hours(1).unwrap(), Some(TimeDelta::try_hours(24).unwrap()), 10), // 1 hour to 1 day -> 10 min
        ];

        let mut conn = match self.db.get_connection() {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get database connection: {}", e);
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())));
            }
        };

        for (start_age, end_age, interval_minutes) in windows {
            let start_time = now - start_age;
            let end_time = end_age.map(|age| now - age);

            if let Err(e) = self.process_time_window(&mut conn, start_time, end_time, interval_minutes) {
                error!("Error processing time window: {}", e);
            }
        }

        Ok(())
    }

    fn process_time_window(
        &self,
        conn: &mut rusqlite::Connection,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
        interval_minutes: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tx = conn.transaction()?;

        // Get all meter IDs
        let meter_ids: Vec<i64> = {
            let mut stmt = tx.prepare("SELECT DISTINCT meter_id FROM meter_readings")?;
            let ids = stmt
                .query_map([], |row| row.get(0))?
                .filter_map(Result::ok)
                .collect();
            stmt.finalize()?;
            ids
        };

        for meter_id in meter_ids {
            if let Err(e) = self.aggregate_meter_data(&tx, meter_id, start_time, end_time, interval_minutes) {
                error!("Error aggregating data for meter {}: {}", meter_id, e);
                return Err(e);
            }
        }

        tx.commit()?;
        Ok(())
    }

    fn aggregate_meter_data(
        &self,
        transaction: &Transaction,
        meter_id: i64,
        start_time: DateTime<Utc>,
        end_time: Option<DateTime<Utc>>,
        interval_minutes: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let start_timestamp = start_time.timestamp();
        let end_timestamp = end_time.map(|t| t.timestamp());

        // Create temporary table for the new aggregated data
        transaction.execute(
            "CREATE TEMPORARY TABLE IF NOT EXISTS temp_aggregated (
                meter_id INTEGER,
                timestamp INTEGER,
                total_power SMALLINT,
                import_power SMALLINT,
                export_power SMALLINT,
                total_kwh REAL
            )",
            [],
        )?;

        // Group data by intervals and calculate averages
        let interval_seconds = interval_minutes * 60;
        let query = format!(
            "INSERT INTO temp_aggregated
             SELECT 
                meter_id,
                (timestamp / ?) * ? as interval_start,
                CAST(AVG(total_power) as INTEGER) as avg_total_power,
                CAST(AVG(import_power) as INTEGER) as avg_import_power,
                CAST(AVG(export_power) as INTEGER) as avg_export_power,
                MAX(total_kwh) as max_total_kwh
             FROM meter_readings
             WHERE meter_id = ? 
             AND timestamp <= ?
             {} 
             GROUP BY meter_id, interval_start",
            if end_timestamp.is_some() {
                "AND timestamp >= ?"
            } else {
                ""
            }
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(interval_seconds),
            Box::new(interval_seconds),
            Box::new(meter_id),
            Box::new(start_timestamp),
        ];

        if let Some(end_ts) = end_timestamp {
            params.push(Box::new(end_ts));
        }

        transaction.execute(&query, rusqlite::params_from_iter(params))?;

        // Delete original data and replace with aggregated data
        let delete_query = format!(
            "DELETE FROM meter_readings 
             WHERE meter_id = ? 
             AND timestamp <= ?
             {}",
            if end_timestamp.is_some() {
                "AND timestamp >= ?"
            } else {
                ""
            }
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![
            Box::new(meter_id),
            Box::new(start_timestamp),
        ];

        if let Some(end_ts) = end_timestamp {
            params.push(Box::new(end_ts));
        }

        transaction.execute(&delete_query, rusqlite::params_from_iter(params))?;

        // Insert aggregated data
        transaction.execute(
            "INSERT INTO meter_readings 
             SELECT * FROM temp_aggregated 
             WHERE meter_id = ?",
            [meter_id],
        )?;

        // Clean up temporary table
        transaction.execute("DELETE FROM temp_aggregated", [])?;

        Ok(())
    }

    pub async fn run(&self) {
        info!("Starting data retention service");

        loop {
            match self.process_retention().await {
                Ok(_) => info!("Successfully processed data retention"),
                Err(e) => error!("Error processing data retention: {}", e),
            }

            // Run every hour
            sleep(Duration::from_secs(3600)).await;
        }
    }
}