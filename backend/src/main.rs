use solarmeter::{
    config::AppConfig,
    database_sync::DatabaseSync,
    meters::{create_meter, MeterReader},
    web_server::WebServer,
    data_retention::RetentionService,
};
use log::{error, info, LevelFilter};

use tokio::{task, time::{sleep, Duration}};
use std::sync::Arc;
use log4rs::{
    append::{
        console::ConsoleAppender,
        file::FileAppender,
        rolling_file::{
            RollingFileAppender,
            policy::compound::{
                CompoundPolicy,
                trigger::size::SizeTrigger,
                roll::fixed_window::FixedWindowRoller,
            },
        },
    },
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use std::path::PathBuf;
use tokio::signal::ctrl_c;

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024;
const LOG_FILES_COUNT: u32 = 5;
const LOG_PATTERN: &str = "{d(%Y-%m-%d %H:%M:%S)} [{l}] {m}{n}";

fn initialize_logging(log_dir: &str, log_level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(log_dir)?;
    let log_path = PathBuf::from(log_dir);

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
        .build();

    let main_log = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
        .build(
            log_path.join("solar_meter.log"),
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(MAX_LOG_SIZE)),
                Box::new(FixedWindowRoller::builder()
                    .build(&format!("{}/solar_meter.{}.log", log_dir, "{}"), LOG_FILES_COUNT)?),
            )),
        )?;

    let error_log = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(LOG_PATTERN)))
        .build(log_path.join("error.log"))?;

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("main_log", Box::new(main_log)))
        .appender(Appender::builder().build("error_log", Box::new(error_log)))
        .build(Root::builder()
            .appender("stdout")
            .appender("main_log")
            .build(log_level))?;

    log4rs::init_config(config)?;
    Ok(())
}

async fn handle_meter(
    mut meter: Box<dyn MeterReader>,
    db_sync: Arc<DatabaseSync>,
    polling_rate: u32,
) {
    // Log initial meter setup
    let meter_name = match meter.get_value().await {
        Ok(reading) => reading.meter_name.clone(),
        Err(e) => {
            error!("Failed to get initial reading from meter: {}", e);
            "Unknown Meter".to_string()
        }
    };

    info!("Started polling loop for meter: {}", meter_name);
    let polling_duration = Duration::from_secs(polling_rate.into());

    loop {
        // Get reading from meter
        let reading_result = meter.get_value().await;
        
        match reading_result {
            Ok(reading) => {
                // Log the successful meter reading
                info!(
                    "Got reading from {}: Power: {:.2}W, Import: {:.2}W, Export: {:.2}W, Total: {:.2}kWh",
                    reading.meter_name, reading.total_power, reading.import_power,
                    reading.export_power, reading.total_kwh
                );
                
                // Store reading in database
                match db_sync.insert_meter_reading(&reading) {
                    Ok(_) => {
                        info!(
                            "Successfully stored reading from {}",
                            reading.meter_name
                        );
                    }
                    Err(e) => {
                        error!(
                            "Failed to insert reading for {}: {}",
                            reading.meter_name,
                            e
                        );
                    }
                }
            }
            Err(e) => {
                error!("Failed to read meter {}: {}", meter_name, e);
                // On error, wait 30 seconds before retrying to avoid spamming logs
                sleep(Duration::from_secs(30)).await;
                continue; // Skip the normal polling delay and retry immediately after error timeout
            }
        }

        // Wait for the next polling interval
        sleep(polling_duration).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(e);
        }
    };
    // Expand the tilde in the log directory path if it exists
    let log_dir = if config.global.log_dir.starts_with("~/") {
        let home = dirs::home_dir()
            .ok_or_else(|| "Could not determine home directory")?;
        home.join(&config.global.log_dir[2..]).to_string_lossy().into_owned()
    } else {
        config.global.log_dir.clone()
    };

    initialize_logging(&log_dir, config.global.log_level.to_level_filter())?;
    info!("Starting solar meter monitoring system");

    info!("Configuration loaded successfully");

    let db_sync = Arc::new(match DatabaseSync::new(&config.global.database_url, config.global.create_database) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            return Err(e);
        }
    });
    info!("Database initialized at {}", config.global.database_url);

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
  
    let web_server = WebServer::new(
        Arc::clone(&db_sync), 
        Some(config.global.bind_address.clone()), 
        shutdown_tx
    );
    let web_server_port = config.global.web_server_port.unwrap_or(8080);
    task::spawn(web_server.run(web_server_port));

    let retention_db = Arc::clone(&db_sync);
    task::spawn(async move {
        let retention_service = RetentionService::new(retention_db);
        retention_service.run().await;
    });

    let mut meter_tasks = Vec::new();
    for (meter_id, meter_config) in &config.meters {
        info!("Creating meter {}: {}", meter_id, meter_config.name);
        
        let meter = create_meter(
            meter_config.name.clone(),
            meter_config.meter_type.clone(),
            meter_config.port.clone(),
            meter_config.baud_rate,
            meter_config.polling_rate,
            meter_config.modbus_address,
            meter_config.timeout,
        ).await;  // Note the .await here

        let db_sync = Arc::clone(&db_sync);
        let polling_rate = meter.get_polling_rate();
        
        meter_tasks.push(task::spawn(async move {
            handle_meter(meter, db_sync, polling_rate).await;
        }));
    }
    info!("Created {} meter polling tasks", meter_tasks.len());

    tokio::select! {
        _ = ctrl_c() => {
            info!("Shutdown signal received from Ctrl+C, stopping gracefully...");
        }
        _ = shutdown_rx => {
            info!("Shutdown signal received from web interface, stopping gracefully...");
        }
    }

    Ok(())
}