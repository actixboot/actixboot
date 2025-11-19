use std::fmt::{Display, Formatter};
use log::LevelFilter;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct LogConfig {
  #[serde(default = "default_log_level")]
  pub level: LogLevel,
}

fn default_log_level() -> LogLevel {
  LogLevel::Info
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
  Off,
  Error,
  Warn,
  Info,
  Debug,
  Trace,
}

impl Display for LogLevel {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      LogLevel::Off => f.write_str("Off"),
      LogLevel::Error => f.write_str("Error"),
      LogLevel::Warn =>  f.write_str("Warn"),
      LogLevel::Info => f.write_str("Info"),
      LogLevel::Debug => f.write_str("Debug"),
      LogLevel::Trace => f.write_str("Trace"),
    }
  }
}

impl Into<log::LevelFilter> for LogLevel {
  fn into(self) -> LevelFilter {
    match self {
      LogLevel::Off => log::LevelFilter::Off,
      LogLevel::Error => log::LevelFilter::Error,
      LogLevel::Warn => log::LevelFilter::Warn,
      LogLevel::Info => log::LevelFilter::Info,
      LogLevel::Debug => log::LevelFilter::Debug,
      LogLevel::Trace => log::LevelFilter::Trace,
    }
  }
}