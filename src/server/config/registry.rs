use crate::repository::Repository;
use crate::service::Service;
use sea_orm::{Database, DatabaseConnection};

pub trait ServiceRegistrator {
  fn register<S>(&mut self)
  where
    S: Service;
}

pub trait RepositoryRegistrator {
  fn register<R>(&mut self)
  where
    R: From<DatabaseConnection> + Repository;
}
