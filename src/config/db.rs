use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
  pub host: String,
  pub port: u16,
  pub username: String,
  pub password: String,
  pub db_name: String,
  pub max_connections: Option<u32>,
  pub min_connections: Option<u32>,
  pub connect_timeout: Option<u64>,
  pub acquire_timeout: Option<u32>,
  pub idle_timeout: Option<u32>,
  pub max_lifetime: Option<u32>,
}
