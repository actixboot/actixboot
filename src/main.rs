use actix_boot::config::SnokeConfig;
use actix_boot::repository::{Repository};
use actix_boot::server::{ApplicationServer, ApplicationServerConfigurer};
use actix_boot::service::{Service};
use actix_boot_repository_macros::repository;
use actix_boot_service_macros::Service;
use actix_web::dev::Payload;
use actix_web::web::{Data, ServiceConfig};
use actix_web::{FromRequest, HttpRequest, Responder, get};
use entity::prelude::Post;
use sea_orm::prelude::*;
use sea_orm::{Database, DatabaseBackend};
use std::future::{Ready, ready};
use std::sync::Arc;

pub mod entity;

#[derive(Debug)]
pub struct User;

pub trait UserRepositoryBase {}

#[derive(Clone)]
pub struct UserRepository {
  db: DatabaseConnection,
}

impl From<DatabaseConnection> for UserRepository {
  fn from(db: DatabaseConnection) -> Self {
    Self { db }
  }
}

impl Repository for UserRepository {
  type Model = User;

  fn find_all(&self) -> Vec<Self::Model> {
    vec![User]
  }
}

#[derive(Service)]
struct UserService {
  user_repository: Arc<UserRepository>,
}

impl UserService {
  pub fn test(&self) -> Vec<User> {
    self.user_repository.find_all()
  }
}

#[derive(Service)]
struct LoginService {
  user_repository: Arc<UserRepository>,
  user_service: Arc<UserService>,
}

impl LoginService {
  pub fn test(&self) -> Vec<User> {
    self.user_service.test()
  }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  ApplicationServer::start(configure_app).await
}

fn configure_app(app: ApplicationServerConfigurer) {
  app.register_service::<LoginService>().configure(|app| {
    app.service(test);
  });
}

#[get("/test")]
async fn test(login_service: Data<LoginService>) -> impl Responder {
  "Test"
}
