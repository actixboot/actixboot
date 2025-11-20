use crate::service::{Service};
use actix_web::web::{Data, ServiceConfig};
use actix_web::{App, HttpServer};
use sea_orm::Database;
use std::sync::Arc;
use crate::di::DIContext;

pub struct ApplicationServer;

impl ApplicationServer {
  pub async fn start<F>(configurer: F) -> std::io::Result<()>
  where
    F: Fn(ApplicationServerConfigurer) + Send + Sync + 'static + Clone,
  {
    let db = Database::connect("postgres://postgres:postgres@localhost:5432/postgres")
      .await
      .unwrap();

    let context = Arc::new(DIContext::new(db));

    HttpServer::new(move || {
      App::new().configure(|app| {
        let app_server_configurer = ApplicationServerConfigurer {
          service_config: app,
          context: context.clone(),
        };

        configurer(app_server_configurer);
      })
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
  }
}

pub struct ApplicationServerConfigurer<'a> {
  service_config: &'a mut ServiceConfig,
  context: Arc<DIContext>,
}

impl ApplicationServerConfigurer<'_> {
  pub fn register_service<S>(self) -> Self
  where
    S: Service,
  {
    let service = self.context.get_service::<S>();
    self.service_config.app_data(Data::from(service));

    self
  }

  pub fn configure<F>(self, function: F) -> Self
  where
    F: Fn(&mut ServiceConfig),
  {
    self.service_config.configure(function);

    self
  }
}
