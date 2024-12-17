use async_trait::async_trait;
use super::MeterReader;
use crate::database_sync::Model;
use chrono::Utc;
use std::time::Duration;

pub struct MockMeter {
    name: String,
    kwh_accumulator: f32,
    last_update: Option<chrono::DateTime<Utc>>,
}

impl MockMeter {
    pub fn new(name: String) -> Self {
        Self {
            name,
            kwh_accumulator: 0.0,
            last_update: None,
        }
    }
}

#[async_trait]
impl MeterReader for MockMeter {
    fn get_polling_rate(&self) -> u32 {
        10
    }
    async fn get_value(&mut self) -> anyhow::Result<Model> {
        let now = Utc::now();
        let total_power = (now.timestamp() as f32 / 3600.0).sin() * 1000.0;
        
        if let Some(last_update) = self.last_update {
            let hours = now.signed_duration_since(last_update).num_milliseconds() as f32 / 3_600_000.0;
            self.kwh_accumulator += total_power * hours;
        }
        self.last_update = Some(now);

        Ok(Model {
            id: 0,
            meter_name: self.name.clone(),
            timestamp: now,
            total_power,
            import_power: total_power.max(0.0),
            export_power: (-total_power).max(0.0),
            total_kwh: self.kwh_accumulator,
        })
    }

    fn get_timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}