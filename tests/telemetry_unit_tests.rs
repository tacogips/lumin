use anyhow::Result;
use log::Level;
use lumin::telemetry::{LogMessage, init, log_with_context};
use std::sync::Mutex;
use std::sync::Once;

// Since we can't easily capture log output in unit tests, these tests focus more on
// ensuring the telemetry functions don't panic and behave as expected

static INIT_TEST: Once = Once::new();
static INIT_RESULT: Mutex<Option<Result<()>>> = Mutex::new(None);

#[test]
fn test_telemetry_init() {
    // Initialize telemetry system
    INIT_TEST.call_once(|| {
        let result = init();
        let mut guard = INIT_RESULT.lock().unwrap();
        *guard = Some(result);
    });

    // Check if initialization succeeded
    let guard = INIT_RESULT.lock().unwrap();
    if let Some(ref result) = *guard {
        assert!(
            result.is_ok(),
            "Telemetry initialization failed: {:?}",
            result
        );
    } else {
        panic!("Initialization result not set");
    }
}

#[test]
fn test_log_with_context_basic() {
    // Ensure telemetry is initialized
    init().ok();

    // Test basic logging without context
    let msg = LogMessage {
        message: "Test log message".to_string(),
        module: "telemetry_test",
        context: None,
    };

    // This should not panic
    log_with_context(Level::Info, msg);
}

#[test]
fn test_log_with_context_detailed() {
    // Ensure telemetry is initialized
    init().ok();

    // Test logging with context at different log levels
    // For each level we need to create a new LogMessage since it doesn't implement Clone

    // Info level
    log_with_context(
        Level::Info,
        LogMessage {
            message: "Test log message with context".to_string(),
            module: "telemetry_test",
            context: Some(vec![
                ("test_key", "test_value".to_string()),
                ("numeric_value", "42".to_string()),
            ]),
        },
    );

    // Debug level
    log_with_context(
        Level::Debug,
        LogMessage {
            message: "Test log message with context".to_string(),
            module: "telemetry_test",
            context: Some(vec![
                ("test_key", "test_value".to_string()),
                ("numeric_value", "42".to_string()),
            ]),
        },
    );

    // Warn level
    log_with_context(
        Level::Warn,
        LogMessage {
            message: "Test log message with context".to_string(),
            module: "telemetry_test",
            context: Some(vec![
                ("test_key", "test_value".to_string()),
                ("numeric_value", "42".to_string()),
            ]),
        },
    );

    // Error level
    log_with_context(
        Level::Error,
        LogMessage {
            message: "Test log message with context".to_string(),
            module: "telemetry_test",
            context: Some(vec![
                ("test_key", "test_value".to_string()),
                ("numeric_value", "42".to_string()),
            ]),
        },
    );

    // Trace level
    log_with_context(
        Level::Trace,
        LogMessage {
            message: "Test log message with context".to_string(),
            module: "telemetry_test",
            context: Some(vec![
                ("test_key", "test_value".to_string()),
                ("numeric_value", "42".to_string()),
            ]),
        },
    );
}

#[test]
fn test_multiple_init_calls() {
    // Multiple init calls should be safe and only initialize once
    let first_result = init();
    let second_result = init();

    assert!(first_result.is_ok());
    assert!(second_result.is_ok());
}
