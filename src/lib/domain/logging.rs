use anyhow::Context;
use chrono::{DateTime, Duration, Local};
use log::{error, info, LevelFilter};
use simplelog::*;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;

use crate::domain::import_result::ImportResults;

const MAX_LOG_AGE: i64 = 7;
const MAX_LOG_COUNT: usize = 100;

pub fn setup_logging() -> anyhow::Result<()> {
    let logs_dir = Path::new("logs");
    fs::create_dir_all(logs_dir).context("failed to create logs directory")?;

    cleanup_old_logs(logs_dir)?;

    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let log_file = logs_dir.join(format!("run_{}.log", timestamp));

    let file_config = ConfigBuilder::new()
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_time_offset_to_local()
        .unwrap_or_else(|builder| builder)
        .build();

    let term_config = ConfigBuilder::new()
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_time_level(LevelFilter::Off)
        .build();

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            term_config,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            file_config,
            OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&log_file)
                .context("failed to open log file")?,
        ),
    ])
    .context("failed to initialize logger")?;

    log::info!("{}", "=".repeat(80));
    log::info!("log session started at {}", timestamp);
    log::info!("{}\n", "=".repeat(80));

    Ok(())
}

fn cleanup_old_logs(logs_dir: &Path) -> anyhow::Result<()> {
    let mut log_files: Vec<_> = fs::read_dir(logs_dir)
        .context("failed to read logs directory")?
        .filter_map(|r| r.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "log"))
        .collect();

    log_files.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    let cutoff_date = Local::now() - Duration::days(MAX_LOG_AGE);

    for entry in log_files.iter().skip(MAX_LOG_COUNT) {
        if let Ok(metadata) = entry.metadata() {
            if let Ok(modified) = metadata.modified() {
                let modified: DateTime<Local> = modified.into();
                if modified < cutoff_date {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[allow(dead_code)]
pub fn log_import_result(result: &ImportResults) -> anyhow::Result<()> {
    info!("\n{}", "=".repeat(80));
    info!("import results summary");
    info!("{}", "=".repeat(80));

    for field in &result.successful {
        info!("✓ successfully imported: {}", field.label);
    }

    for field in &result.failed {
        error!(
            "✗ failed to import: {} ({})",
            field.label,
            field.error.as_ref().unwrap_or(&"unknown error".to_string())
        );
    }

    info!("\n{}", "=".repeat(80));
    Ok(())
}
