use std::fs;
use std::fs::OpenOptions;
use std::path::Path;

use anyhow::Context;
use chrono::{DateTime, Duration, Local};
use log::LevelFilter;
use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, TermLogger, TerminalMode, WriteLogger,
};

const MAX_LOG_AGE: i64 = 7;
const MAX_LOG_COUNT: usize = 100;
/// Logs cleanup keeps, leaving room for the one the run creates next.
const KEPT_BEFORE_RUN: usize = MAX_LOG_COUNT.saturating_sub(1);

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
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "log"))
        .collect();

    log_files.sort_by(|a, b| {
        let a_time = a.metadata().and_then(|m| m.modified()).ok();
        let b_time = b.metadata().and_then(|m| m.modified()).ok();
        b_time.cmp(&a_time)
    });

    let cutoff_date = Local::now() - Duration::days(MAX_LOG_AGE);

    // Newest first, so everything past KEPT_BEFORE_RUN is surplus.
    for (index, entry) in log_files.iter().enumerate() {
        let too_old = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .is_ok_and(|modified| DateTime::<Local>::from(modified) < cutoff_date);
        if index >= KEPT_BEFORE_RUN || too_old {
            let _ = fs::remove_file(entry.path());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{Duration as StdDuration, SystemTime};

    use super::*;

    const DAY: StdDuration = StdDuration::from_secs(60 * 60 * 24);

    /// An empty directory unique to `name` and this process.
    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("halo_logs_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Writes an empty file last modified `age` ago.
    fn write_aged(path: &Path, age: StdDuration) {
        std::fs::write(path, "").unwrap();
        std::fs::File::options()
            .write(true)
            .open(path)
            .unwrap()
            .set_modified(SystemTime::now() - age)
            .unwrap();
    }

    #[test]
    fn cleanup_deletes_a_log_past_the_age_even_under_the_count() {
        let dir = scratch_dir("age");
        write_aged(&dir.join("fresh.log"), StdDuration::ZERO);
        let old = dir.join("old.log");
        write_aged(&old, DAY * (MAX_LOG_AGE as u32 + 1));
        let other = dir.join("notes.txt");
        write_aged(&other, DAY * (MAX_LOG_AGE as u32 + 1));

        cleanup_old_logs(&dir).unwrap();

        let (old_gone, fresh_kept, other_kept) = (
            !old.exists(),
            dir.join("fresh.log").exists(),
            other.exists(),
        );
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(old_gone, "the old log goes");
        assert!(fresh_kept, "the fresh log stays");
        assert!(other_kept, "a file that is not a log stays");
    }

    #[test]
    fn cleanup_keeps_only_the_newest_logs_up_to_the_count() {
        let dir = scratch_dir("count");
        for i in 0..MAX_LOG_COUNT {
            write_aged(
                &dir.join(format!("fresh_{i}.log")),
                StdDuration::from_secs(i as u64),
            );
        }
        // Still fresh, but older than every other log.
        let oldest = dir.join("oldest.log");
        write_aged(&oldest, StdDuration::from_secs(60 * 60));

        cleanup_old_logs(&dir).unwrap();

        let remaining = std::fs::read_dir(&dir).unwrap().count();
        let oldest_gone = !oldest.exists();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(oldest_gone, "the log beyond the count goes");
        // The run's own log comes next, which makes MAX_LOG_COUNT.
        assert_eq!(remaining, MAX_LOG_COUNT - 1);
    }
}
