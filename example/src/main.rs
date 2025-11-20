use std::sync::Arc;
use actix_boot::server::ApplicationServer;
use actix_web::web::Data;
use actix_web::{get, Responder};
use actix_boot::di::GetOrCreate;
use actix_boot::service::derive::Service;

pub mod entity;

#[derive(Service)]
pub struct LogService;

impl LogService {
  pub fn log(&self, message: String) -> String {
    println!("{}", &message);
    message
  }
}

#[derive(Service)]
pub struct TestService {
  log_service: Arc<LogService>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  ApplicationServer::start(|app, _| {
    app.configure(|app, ctx| {
      app.app_data(TestService::get_or_create(ctx));
      app.service(test);
    });
  }).await
}

#[get("/test")]
async fn test(test_service: Data<TestService>) -> impl Responder {
  test_service.log_service.log("hello".to_string())
}
