use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logger() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "cvd_trader_rust=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Simple logging macros for testing
#[macro_export]
macro_rules! log {
    (info, $($arg:tt)*) => {
//         println!("[INFO] {}", format_args!($($arg)*));
    };
    (warn, $($arg:tt)*) => {
//         println!("[WARN] {}", format_args!($($arg)*));
    };
    (error, $($arg:tt)*) => {
// //         eprintln!("[ERROR] {}", format_args!($($arg)*));
    };
    (debug, $($arg:tt)*) => {
//         println!("[DEBUG] {}", format_args!($($arg)*));
    };
}

// Re-export for convenience
pub use log;