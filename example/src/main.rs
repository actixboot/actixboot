use actix_boot::di::GetOrCreate;
use actix_boot::repository::Repository;
use actix_boot::repository::macros::repository;
use actix_boot::server::ApplicationServer;
use actix_web::web::{Data, Json};
use actix_web::{Responder, get};
use entity::post;
use sea_orm::sea_query::ExprTrait;
use sea_orm::{ColumnTrait, EntityOrSelect, EntityTrait, PaginatorTrait, QueryFilter};

pub mod entity;

#[repository(post)]
pub trait PostRepositoryBase {
  async fn find_by_text_and_title(
    &self,
    text: &str,
    title: &str,
  ) -> Result<Option<post::Model>, sea_orm::DbErr>;
}

/*
  find_all_*
  delete_all
  find_*
  delete_*
  exists_*
  count_*
*/

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
  Json(
    post_repository
      .find_by_text_and_title("haha", "test")
      .await
      .unwrap(),
  )
}
