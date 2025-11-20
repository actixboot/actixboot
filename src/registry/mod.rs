use crate::server::config::ApplicationServerConfigurer;

pub trait Registry {
  fn register_all(context: &mut ApplicationServerConfigurer);
}