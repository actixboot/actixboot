use std::any::Any;

pub mod macros;

type SeaResult<T> = std::result::Result<T, sea_orm::DbErr>;

pub trait Repository: Any + Send + Sync {
  type Model;

  fn find_all(&self) -> impl Future<Output = SeaResult<Vec<Self::Model>>> + Send;

  fn find(&self, id: i32) -> impl Future<Output = SeaResult<Option<Self::Model>>> + Send;

  fn exists(&self, id: i32) -> impl Future<Output = SeaResult<bool>> + Send;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Paginator<T> {
  pub items: Vec<T>,
  pub page: u64,
  pub per_page: u64,
  pub total_items: u64,
  pub total_pages: u64,
}

impl<T> Paginator<T> {
  pub fn has_next(&self) -> bool {
    self.page < self.total_pages
  }

  pub fn has_prev(&self) -> bool {
    self.page > 1
  }

  pub fn next_page(&self) -> Option<u64> {
    if self.has_next() {
      Some(self.page + 1)
    } else {
      None
    }
  }

  pub fn prev_page(&self) -> Option<u64> {
    if self.has_prev() {
      Some(self.page - 1)
    } else {
      None
    }
  }

  pub fn is_first_page(&self) -> bool {
    self.page == 1
  }

  pub fn is_last_page(&self) -> bool {
    self.page == self.total_pages || self.total_pages == 0
  }
}
