use m_core::util::file;
use std::fs;
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub fn setup_logger() -> anyhow::Result<WorkerGuard> {
    let log_path = file::cd().join("app.log");
    if file::cd().join("app.log").exists() {
        fs::remove_file(&log_path)?;
    }
    let file_appender = tracing_appender::rolling::never(file::cd(), "app.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = fmt::layer().with_ansi(false).with_writer(non_blocking);
    let console_layer = fmt::layer().with_ansi(true);

    tracing_subscriber::registry()
        .with(file_layer)
        .with(console_layer)
        .init();

    info!("Logging to {}", log_path.display());

    Ok(guard)
}
