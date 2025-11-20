use crate::di::{DIContext};
use std::any::Any;
use crate::server::config::ApplicationServerConfigurer;

pub mod derive;

pub trait Service: Any + Send + Sync {
  fn new_service(context: &DIContext) -> Self;
}
