use std::env;
use regex::{Captures, Regex};
use serde::Deserialize;
use crate::config::db::{DbConfig};
use crate::config::log::LogConfig;
use crate::config::server::ServerConfig;

pub mod db;
pub mod server;
pub mod log;

#[derive(Debug, Clone, Deserialize)]
pub struct SnokeConfig {
  pub db: Option<DbConfig>,
  pub server: ServerConfig,
  pub log: LogConfig,
}

impl SnokeConfig {
  pub fn load_from_yaml() -> Self {
    let config = Self::parse_config();
    serde_yaml::from_str(&config).expect("Failed to parse YAML config")
  }

  pub fn parse_config() -> String {
    let config_file = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/config/app.yaml"));
    let re = Regex::new(r"\$\{([^:}]+)(?::([^}]*))?}").unwrap(); // ${ENV:default}

    re.replace_all(config_file, |caps: &Captures| {
      let var_name = &caps[1];
      let default_value = caps.get(2).map(|m| m.as_str());

      env::var(var_name)
        .unwrap_or_else(|_| default_value.expect(&format!("You need to specify {}", var_name)).to_string())
    }).to_string()
  }
}