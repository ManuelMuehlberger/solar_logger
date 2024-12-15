use chrono::Utc;
use rand::Rng;
use super::MeterReader;
use crate::database_sync::Model;

pub struct MockMeter {
    name: String,
    min_power: f64,
    max_power: f64,
    power_variation: f64,
    kwh_accumulator: f64,
}

impl MockMeter {
    pub fn new(name: String, min_power: f64, max_power: f64, power_variation: f64) -> Self {
        Self {
            name,
            min_power,
            max_power,
            power_variation,
            kwh_accumulator: 0.0,
        }
    }

    fn generate_power(&self) -> f64 {
        let mut rng = rand::thread_rng();
        let base = rng.gen_range(self.min_power..=self.max_power);
        base + (rng.gen_range(-self.power_variation..=self.power_variation))
    }
}

impl MeterReader for MockMeter {
    fn get_value(&self) -> Result<Model, Box<dyn std::error::Error>> {
        let total_power = self.generate_power();
        let import_power = if total_power > 0.0 { total_power } else { 0.0 };
        let export_power = if total_power < 0.0 { -total_power } else { 0.0 };
        let kwh_delta = total_power / 3600.0;
        Ok(Model {
            id: 0,
            meter_name: self.name.clone(),
            timestamp: Utc::now(),
            total_power,
            import_power,
            export_power,
            total_kwh: self.kwh_accumulator + kwh_delta,
        })
    }
}