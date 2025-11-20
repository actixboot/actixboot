use std::sync::Arc;
use actix_boot::server::ApplicationServer;
use actix_web::web::Data;
use actix_web::{get, Responder};
use actix_boot::repository::macros::repository;
use actix_boot::service::derive::Service;

pub mod entity;

#[repository(entity = entity::Post, model = entity::post::Model)]
pub trait PostRepositoryBase {}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  ApplicationServer::start(|app, _| {
    app.configure(|app, ctx| {
      app.service(test);
    });
  }).await
}

#[get("/test")]
async fn test() -> impl Responder {
  "hello"
}
