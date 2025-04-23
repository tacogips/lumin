//! Telemetry and logging configuration using OpenTelemetry.
//!
//! This module provides utilities for setting up OpenTelemetry-based logging
//! with stderr output for console visibility, as well as structured telemetry
//! data collection (traces, metrics, logs).

use anyhow::Result;
use log::{Level, error, info, warn};
use opentelemetry::sdk::trace::Sampler;
use opentelemetry_sdk::{Resource, logs};
use opentelemetry_stdout::LogExporter;
use std::sync::Once;

static INIT: Once = Once::new();

/// Log message with context for OpenTelemetry
pub struct LogMessage {
    /// The message to log
    pub message: String,

    /// The module where the log originated
    pub module: &'static str,

    /// Optional key-value pairs of additional context
    pub context: Option<Vec<(&'static str, String)>>,
}

/// Initialize OpenTelemetry logging with stderr output
///
/// This function sets up OpenTelemetry logging with a stderr exporter
/// and configures the global default logger provider.
///
/// # Returns
///
/// A Result indicating success or failure of the initialization
pub fn init() -> Result<()> {
    let mut result = Ok(());

    INIT.call_once(|| {
        match setup_telemetry() {
            Ok(_) => {
                // Initialize successful
                info!("OpenTelemetry logging initialized with stderr output");
            }
            Err(e) => {
                // Cannot use logging yet since it failed to initialize
                eprintln!("Failed to initialize OpenTelemetry logging: {}", e);
                result = Err(e);
            }
        }
    });

    result
}

/// Log a message with the given level and context
///
/// # Arguments
///
/// * `level` - The log level to use
/// * `msg` - The log message with context
///
/// # Example
///
/// ```
/// use lumin::telemetry::{log_with_context, LogMessage};
/// use log::Level;
///
/// log_with_context(
///     Level::Info,
///     LogMessage {
///         message: "File processed successfully".to_string(),
///         module: "search",
///         context: Some(vec![
///             ("file_path", "/path/to/file.txt".to_string()),
///             ("matches", "5".to_string()),
///         ]),
///     }
/// );
/// ```
pub fn log_with_context(level: Level, msg: LogMessage) {
    match level {
        Level::Error => {
            error!(target: msg.module, "{}", format_context(&msg));
        }
        Level::Warn => {
            warn!(target: msg.module, "{}", format_context(&msg));
        }
        Level::Info => {
            info!(target: msg.module, "{}", format_context(&msg));
        }
        Level::Debug => {
            log::debug!(target: msg.module, "{}", format_context(&msg));
        }
        Level::Trace => {
            log::trace!(target: msg.module, "{}", format_context(&msg));
        }
    }
}

/// Format a log message with its context for display
fn format_context(msg: &LogMessage) -> String {
    if let Some(context) = &msg.context {
        let context_str = context
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");

        format!("{} [{}]", msg.message, context_str)
    } else {
        msg.message.clone()
    }
}

/// Set up the OpenTelemetry logging pipeline
fn setup_telemetry() -> Result<()> {
    // Create a stderr exporter for OpenTelemetry logs
    let exporter = LogExporter::default();

    // Configure the logger provider with the stderr exporter
    let provider = logs::LoggerProvider::builder()
        .with_config(logs::config().with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", "lumin"),
            opentelemetry::KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ])))
        .with_simple_exporter(exporter)
        .build();

    // Set as the global provider
    opentelemetry::global::set_logger_provider(provider.clone());

    // Initialize the log crate integration
    let logger = provider.logger(opentelemetry_sdk::logs::LoggerConfig::default());

    log::set_logger(Box::leak(Box::new(logger)))?;
    log::set_max_level(log::LevelFilter::Info);

    Ok(())
}
