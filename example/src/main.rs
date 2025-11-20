use crate::entity::prelude::Post as PostEntity;
use actix_boot::di::GetOrCreate;
use actix_boot::repository::Repository;
use actix_boot::repository::macros::repository;
use actix_boot::server::ApplicationServer;
use actix_web::web::{Data, Json};
use actix_web::{Responder, get};
use entity::post;
use post::Model as PostModel;
use sea_orm::{EntityTrait, PaginatorTrait};

pub mod entity;

#[repository(entity = PostEntity, model = PostModel)]
pub trait PostRepositoryBase {}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
  ApplicationServer::start(|app, _| {
    app.configure(|app, ctx| {
      app.app_data(PostRepository::get_or_create(ctx));
      app.service(test);
    });
  })
  .await
}

#[get("/test")]
async fn test(post_repository: Data<PostRepository>) -> impl Responder {
  Json(post_repository.exists(2).await.unwrap())
}
