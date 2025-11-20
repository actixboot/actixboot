use std::sync::Arc;
use actix_web::web::{Data, ServiceConfig};
use sea_orm::{Database, DatabaseConnection};
use crate::di::DIContext;
use crate::repository::Repository;
use crate::server::config::registry::{RepositoryRegistrator, ServiceRegistrator};
use crate::service::Service;

pub mod registry;

pub struct ApplicationServerConfigurer<'a> {
  pub(crate) service_config: &'a mut ServiceConfig,
  pub(crate) context: Arc<DIContext>,
}

impl ServiceRegistrator for ApplicationServerConfigurer<'_> {
  fn register<S>(&mut self)
  where
    S: Service
  {
    let service = self.context.get_service::<S>();
    self.service_config.app_data(Data::from(service));
  }
}

impl RepositoryRegistrator for ApplicationServerConfigurer<'_> {
  fn register<R>(&mut self)
  where
    R: From<DatabaseConnection> + Repository
  {
    let repository = self.context.get_repository::<R>();
    self.service_config.app_data(Data::from(repository));
  }
}

impl ApplicationServerConfigurer<'_> {
  pub fn configure<F>(&mut self, function: F)
  where
    F: Fn(&mut ServiceConfig),
  {
    self.service_config.configure(function);
  }
}