//! Telemetry setup: installs the single tracing subscriber for the application.

use std::env;

use tracing::warn;
use tracing_subscriber::{EnvFilter, fmt};

const DEFAULT_FILTER: &str = "info";

// NOTE: an unparseable RUST_LOG degrades to the default filter instead of aborting startup
pub fn init() {
    let raw = env::var("RUST_LOG").ok();
    let default = || EnvFilter::new(DEFAULT_FILTER);

    let (filter, invalid) = match raw.as_deref() {
        Some(value) => match EnvFilter::try_new(value) {
            Ok(filter) => (filter, None),
            // INFO: no subscriber is installed yet, so the fallback warn is logged after init
            Err(err) => (default(), Some((value, err))),
        },
        None => (default(), None),
    };

    fmt().compact().with_env_filter(filter).init();

    if let Some((value, err)) = invalid {
        warn!(value = %value, error = %err, "invalid RUST_LOG; using default filter {DEFAULT_FILTER}");
    }
}
