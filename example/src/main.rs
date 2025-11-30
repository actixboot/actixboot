use actix_web::{get, main, Responder};
use actix_web::web::{Data, Json};
use actix_boot::di::GetOrCreate;
use actix_boot::repository::macros::repository;
use actix_boot::server::ApplicationServer;
use entity::post;
use sea_orm::entity::prelude::*;

pub mod entity;

#[repository(post)]
pub trait PostRepositoryBase {
  // Find single record with multiple filters
  async fn find_by_text_and_title(
    &self,
    text: &str,
    title: &str,
  ) -> Result<Option<post::Model>, sea_orm::DbErr>;

  // Find all records with a filter
  async fn find_all_by_text(
    &self,
    text: &str,
  ) -> Result<Vec<post::Model>, sea_orm::DbErr>;

  // Count records matching criteria
  async fn count_by_title(
    &self,
    title: &str,
  ) -> Result<u64, sea_orm::DbErr>;

  // Delete records
  async fn delete_by_text(
    &self,
    text: &str,
  ) -> Result<sea_orm::DeleteResult, sea_orm::DbErr>;

  // Test with comparison filters
  async fn find_by_id_gt(
    &self,
    id: i32,
  ) -> Result<Option<post::Model>, sea_orm::DbErr>;

  // Find with LIKE filter
  async fn find_all_by_text_like(
    &self,
    text: &str,
  ) -> Result<Vec<post::Model>, sea_orm::DbErr>;

  // Find with multiple filters on one column (between range)
  async fn find_by_id_gte_lte(
    &self,
    min_id: i32,
    max_id: i32,
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

#[main]
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
