use crate::di::DIContext;
use std::any::Any;

pub trait Service: Any + Send + Sync {
  fn new_service(context: &DIContext) -> Self;
}
