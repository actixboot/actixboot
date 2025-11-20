use std::ops::{Deref, DerefMut};
use actix_web::web::{Data, ServiceConfig};
use actix_web::{App, HttpServer};
use sea_orm::Database;
use std::sync::Arc;
use crate::di::DIContext;
use crate::server::config::ApplicationServerConfigurer;

pub mod config;

pub struct ApplicationServer;

pub struct Application<'a> {
  service_config: &'a mut ServiceConfig,
  ctx: &'a DIContext,
}

impl Application<'_> {
  pub fn configure<F>(&mut self, function: F) where F: FnOnce(&mut Application, &DIContext) {
    function(self, self.ctx);
  }

  pub fn ctx(&self) -> &DIContext {
    self.ctx
  }
}

impl Deref for Application<'_> {
  type Target = ServiceConfig;

  fn deref(&self) -> &Self::Target {
    self.service_config
  }
}

impl<> DerefMut for Application<'_> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.service_config
  }
}

impl ApplicationServer {
  pub async fn start<F>(configurer: F) -> std::io::Result<()>
  where
    F: Fn(&mut Application, &DIContext) + Send + Sync + 'static + Clone,
  {
    let db = Database::connect("postgres://postgres:postgres@localhost:5432/postgres")
      .await
      .unwrap();

    let context = Arc::new(DIContext::new(db));

    HttpServer::new(move || {
      App::new().configure(|cfg| {
        let mut app = Application {
          service_config: cfg,
          ctx: context.as_ref()
        };

        configurer(&mut app, context.as_ref());
      })
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
  }
}
