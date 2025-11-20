use sea_orm::prelude::DatabaseConnection;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub mod macros;

pub trait Repository: Any + Send + Sync {
  type Model;

  fn find_all(&self) -> Vec<Self::Model>;
}
