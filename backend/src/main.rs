use solarmeter::{
    config::AppConfig,
    database_sync::DatabaseSync,
    meters::{create_meter, MeterReader},
    web_server::WebServer
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

fn initialize_logging(log_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
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
            .build(LevelFilter::Info))?;

    log4rs::init_config(config)?;
    Ok(())
}

async fn handle_meter(
    mut meter: Box<dyn MeterReader>,
    db_sync: Arc<DatabaseSync>,
    polling_rate: u32,
) {
    let meter_name = match meter.get_value().await {
        Ok(reading) => reading.meter_name.clone(),
        Err(_) => "Unknown Meter".to_string(),
    };

    info!("Started polling loop for meter: {}", meter_name);
    let polling_duration = Duration::from_secs(polling_rate.into());

    loop {
        match meter.get_value().await {
            Ok(reading) => {
                match db_sync.insert_meter_reading(&reading) {
                    Ok(_) => {
                        info!(
                            "Reading from {}: Power: {:.2}W, Import: {:.2}W, Export: {:.2}W, Total: {:.2}kWh",
                            reading.meter_name, reading.total_power, reading.import_power,
                            reading.export_power, reading.total_kwh
                        );
                    }
                    Err(e) => {
                        error!("Failed to insert reading for {}: {}", reading.meter_name, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to read meter {}: {}", meter_name, e);
                sleep(Duration::from_secs(30)).await;
            }
        }

        sleep(polling_duration).await;
    }
}

async fn health_check(port: u16) {
    use warp::Filter;
    
    let health = warp::path!("health")
        .map(|| "OK");
    
    info!("Starting health check endpoint on port {}", port);
    warp::serve(health)
        .run(([127, 0, 0, 1], port))
        .await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    initialize_logging("~/log/solar_meter")?;
    info!("Starting solar meter monitoring system");

    let config = match AppConfig::from_file("src/config.toml") {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            return Err(e);
        }
    };
    info!("Configuration loaded successfully");

    let db_sync = Arc::new(match DatabaseSync::new(&config.global.database_url, config.global.create_database) {
        Ok(db) => db,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            return Err(e);
        }
    });
    info!("Database initialized at {}", config.global.database_url);
  
    let web_server = WebServer::new(Arc::clone(&db_sync), Some(config.global.bind_address));
    let web_server_port = config.global.web_server_port.unwrap_or(8080);
    task::spawn(web_server.run(web_server_port));


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
        );

        let db_sync = Arc::clone(&db_sync);
        let polling_rate = meter.get_polling_rate();
        
        meter_tasks.push(task::spawn(async move {
            handle_meter(meter, db_sync, polling_rate).await;
        }));
    }
    info!("Created {} meter polling tasks", meter_tasks.len());

    match ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, stopping gracefully...");
        }
        Err(e) => error!("Error waiting for shutdown signal: {}", e),
    }

    Ok(())
}