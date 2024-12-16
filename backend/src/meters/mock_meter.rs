use chrono::Utc;
use rand::Rng;
use async_trait::async_trait;
use super::MeterReader;
use crate::database_sync::Model;
use std::time::Duration;


pub struct MockMeter {
    name: String,
    min_power: f32,
    max_power: f32,
    power_variation: f32,
    kwh_accumulator: f32,
    last_update: Option<chrono::DateTime<Utc>>,
}

impl MockMeter {
    pub fn new(name: String, min_power: f32, max_power: f32, power_variation: f32) -> Self {
        Self {
            name,
            min_power,
            max_power,
            power_variation,
            kwh_accumulator: 0.0,
            last_update: None,
        }
    }

    fn generate_power(&self) -> f32 {
        let mut rng = rand::thread_rng();
        let base = rng.gen_range(self.min_power..=self.max_power);
        base + (rng.gen_range(-self.power_variation..=self.power_variation))
    }
}

#[async_trait]
impl MeterReader for MockMeter {
    async fn get_value(&mut self) -> Result<Model, Box<dyn std::error::Error>> {
        let now = Utc::now();
        let total_power = self.generate_power();
        
        // Update kwh accumulator based on time elapsed
        if let Some(last_update) = self.last_update {
            let duration = now.signed_duration_since(last_update);
            let hours = duration.num_milliseconds() as f32 / (1000.0 * 3600.0);
            self.kwh_accumulator += total_power * hours;
        }
        self.last_update = Some(now);

        let import_power = if total_power > 0.0 { total_power } else { 0.0 };
        let export_power = if total_power < 0.0 { -total_power } else { 0.0 };
        
        Ok(Model {
            id: 0,
            meter_name: self.name.clone(),
            timestamp: now,
            total_power,
            import_power,
            export_power,
            total_kwh: self.kwh_accumulator,
        })
    }

    fn get_timeout(&mut self) -> Duration {
        Duration::new(5, 0)
    }
}