use crate::repository::Repositories;
use crate::service::{Service, Services};
use actix_web::web::{Data, ServiceConfig};
use actix_web::{App, HttpServer};
use sea_orm::Database;
use std::sync::Arc;

pub struct ApplicationServer;

impl ApplicationServer {
  pub async fn start<F>(configurer: F) -> std::io::Result<()>
  where
    F: Fn(ApplicationServerConfigurer) + Send + Sync + 'static + Clone,
  {
    let db = Database::connect("postgres://postgres:postgres@localhost:5432/postgres")
      .await
      .unwrap();
    let repositories = Arc::new(Repositories::new(db));
    let services = Arc::new(Services::new(repositories.clone()));

    HttpServer::new(move || {
      App::new().configure(|app| {
        let app_server_configurer = ApplicationServerConfigurer {
          service_config: app,
          services: services.clone(),
          repositories: repositories.clone(),
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
  services: Arc<Services>,
  repositories: Arc<Repositories>,
}

impl ApplicationServerConfigurer<'_> {
  pub fn register_service<S>(self) -> Self
  where
    S: Service,
  {
    let service = self.services.get_service::<S>();
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
