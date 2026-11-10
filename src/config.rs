//! Environment-based application configuration.

use std::env;
use std::fmt;
use std::num::NonZeroU32;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;
// NOTE: conservative free-tier defaults; raise before launch if justified by traffic
const DEFAULT_RATE_LIMIT_PER_SECOND: u64 = 5;
const DEFAULT_RATE_LIMIT_BURST: u32 = 20;

#[derive(Debug)]
pub struct Config {
    host: String,
    port: u16,
    rate_limit: RateLimitConfig,
    trust_proxy_headers: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = match env::var("HOST") {
            Ok(host) => host,
            Err(env::VarError::NotPresent) => DEFAULT_HOST.to_owned(),
            Err(env::VarError::NotUnicode(os)) => {
                return Err(ConfigError::NotUnicode {
                    variable: "HOST",
                    raw: os.to_string_lossy().into_owned(),
                });
            }
        };

        // NOTE: explicitly empty HOST falls back to the default
        let host = if host.trim().is_empty() {
            DEFAULT_HOST.to_owned()
        } else {
            host
        };

        let port = match env::var("PORT") {
            Ok(raw) => parse_port(&raw)?,
            Err(env::VarError::NotPresent) => DEFAULT_PORT,
            Err(env::VarError::NotUnicode(os)) => {
                return Err(ConfigError::NotUnicode {
                    variable: "PORT",
                    raw: os.to_string_lossy().into_owned(),
                });
            }
        };

        let per_second = match env::var("RATE_LIMIT_PER_SECOND") {
            Ok(raw) => parse_non_zero(&raw, "RATE_LIMIT_PER_SECOND")?,
            Err(env::VarError::NotPresent) => {
                NonZeroU32::try_from(DEFAULT_RATE_LIMIT_PER_SECOND).expect("non-zero constant")
            }
            Err(env::VarError::NotUnicode(os)) => {
                return Err(ConfigError::NotUnicode {
                    variable: "RATE_LIMIT_PER_SECOND",
                    raw: os.to_string_lossy().into_owned(),
                });
            }
        };

        let burst = match env::var("RATE_LIMIT_BURST") {
            Ok(raw) => parse_non_zero(&raw, "RATE_LIMIT_BURST")?,
            Err(env::VarError::NotPresent) => {
                NonZeroU32::new(DEFAULT_RATE_LIMIT_BURST).expect("non-zero constant")
            }
            Err(env::VarError::NotUnicode(os)) => {
                return Err(ConfigError::NotUnicode {
                    variable: "RATE_LIMIT_BURST",
                    raw: os.to_string_lossy().into_owned(),
                });
            }
        };

        let trust_proxy_headers = match env::var("TRUST_PROXY_HEADERS") {
            Ok(raw) => parse_bool(&raw, "TRUST_PROXY_HEADERS")?,
            Err(env::VarError::NotPresent) => false,
            Err(env::VarError::NotUnicode(os)) => {
                return Err(ConfigError::NotUnicode {
                    variable: "TRUST_PROXY_HEADERS",
                    raw: os.to_string_lossy().into_owned(),
                });
            }
        };

        Ok(Self {
            host,
            port,
            rate_limit: RateLimitConfig { per_second, burst },
            trust_proxy_headers,
        })
    }

    // NOTE: bind_addr() is the single place host and port combine
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn rate_limit(&self) -> &RateLimitConfig {
        &self.rate_limit
    }

    pub fn trust_proxy_headers(&self) -> bool {
        self.trust_proxy_headers
    }
}

#[derive(Debug)]
pub struct RateLimitConfig {
    per_second: NonZeroU32,
    burst: NonZeroU32,
}

impl RateLimitConfig {
    pub fn per_second(&self) -> NonZeroU32 {
        self.per_second
    }

    pub fn burst(&self) -> NonZeroU32 {
        self.burst
    }
}

#[derive(Debug)]
pub enum ConfigError {
    /// PORT was set explicitly but is not a valid port number.
    InvalidPort { raw: String },
    /// A rate-limit or proxy variable was set explicitly but is not a valid value.
    InvalidValue {
        variable: &'static str,
        raw: String,
        reason: &'static str,
    },
    /// HOST or PORT contains bytes that are not valid Unicode.
    NotUnicode { variable: &'static str, raw: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPort { raw } => write!(f, "PORT is not a valid port number: {raw:?}"),
            Self::InvalidValue {
                variable,
                raw,
                reason,
            } => write!(f, "{variable} is invalid: {reason} ({raw:?})"),
            Self::NotUnicode { variable, raw } => {
                write!(f, "{variable} contains invalid Unicode: {raw:?}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

fn parse_port(raw: &str) -> Result<u16, ConfigError> {
    raw.parse().map_err(|_| ConfigError::InvalidPort {
        raw: raw.to_owned(),
    })
}

fn parse_non_zero(raw: &str, variable: &'static str) -> Result<NonZeroU32, ConfigError> {
    raw.parse().map_err(|_| ConfigError::InvalidValue {
        variable,
        raw: raw.to_owned(),
        reason: "must be a positive integer",
    })
}

fn parse_bool(raw: &str, variable: &'static str) -> Result<bool, ConfigError> {
    raw.parse().map_err(|_| ConfigError::InvalidValue {
        variable,
        raw: raw.to_owned(),
        reason: "must be `true` or `false`",
    })
}

#[cfg(test)]
impl Config {
    // NOTE: deterministic construction for router tests; bypasses env entirely
    pub(crate) fn for_test() -> Self {
        Self {
            host: "127.0.0.1".to_owned(),
            port: 5000,
            rate_limit: RateLimitConfig {
                per_second: NonZeroU32::MIN,
                burst: NonZeroU32::MIN,
            },
            trust_proxy_headers: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_port_parses() {
        assert_eq!(parse_port("5000").ok(), Some(5000));
    }

    #[test]
    fn invalid_port_reports_raw_value() {
        match parse_port("not-a-port") {
            Err(ConfigError::InvalidPort { raw }) => assert_eq!(raw, "not-a-port"),
            result => panic!("expected invalid port error, got {result:?}"),
        }
    }

    #[test]
    fn port_out_of_range_is_rejected() {
        assert!(parse_port("70000").is_err());
    }

    #[test]
    fn zero_rate_limit_is_rejected() {
        assert!(parse_non_zero("0", "RATE_LIMIT_PER_SECOND").is_err());
    }

    #[test]
    fn negative_rate_limit_is_rejected() {
        assert!(parse_non_zero("-1", "RATE_LIMIT_BURST").is_err());
    }

    #[test]
    fn non_numeric_rate_limit_is_rejected() {
        assert!(parse_non_zero("many", "RATE_LIMIT_PER_SECOND").is_err());
    }

    #[test]
    fn valid_positive_rate_limit_parses() {
        assert!(parse_non_zero("5", "RATE_LIMIT_PER_SECOND").is_ok());
    }

    #[test]
    fn invalid_bool_is_rejected() {
        assert!(parse_bool("yes", "TRUST_PROXY_HEADERS").is_err());
    }

    #[test]
    fn valid_bool_parses() {
        assert_eq!(parse_bool("true", "TRUST_PROXY_HEADERS").ok(), Some(true));
    }
}
