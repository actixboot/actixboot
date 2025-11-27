use std::any::Any;

use sea_orm::{Selector, SelectorTrait, sea_query::SimpleExpr};

pub mod macros;

type SeaResult<T> = std::result::Result<T, sea_orm::DbErr>;

pub trait Repository: Any + Send + Sync {
  type Model;

  fn find_all(&self) -> impl Future<Output = SeaResult<Vec<Self::Model>>> + Send;

  fn find(&self, id: i32) -> impl Future<Output = SeaResult<Option<Self::Model>>> + Send;

  fn exists(&self, id: i32) -> impl Future<Output = SeaResult<bool>> + Send;
}
