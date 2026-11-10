use std::env;
use std::fmt;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3000;

#[derive(Debug)]
pub struct Config {
    host: String,
    port: u16,
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

        Ok(Self { host, port })
    }

    // NOTE: bind_addr() is the single place host and port combine
    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    /// PORT was set explicitly but is not a valid port number.
    InvalidPort { raw: String },
    /// HOST or PORT contains bytes that are not valid Unicode.
    NotUnicode { variable: &'static str, raw: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPort { raw } => write!(f, "PORT is not a valid port number: {raw:?}"),
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
}
