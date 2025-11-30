use actix_boot::di::GetOrCreate;
use actix_boot::repository::macros::repository;
use actix_boot::server::ApplicationServer;
use actix_web::web::{Data, Json};
use actix_web::{Responder, get, main};
use entity::post;
use sea_orm::entity::prelude::*;

pub mod entity;

#[repository(post)]
pub trait PostRepositoryBase {
  async fn find_by_text_and_title(
    &self,
    text: &str,
    title: &str,
  ) -> Result<Option<post::Model>, sea_orm::DbErr>;

  async fn find_all_by_text(&self, text: &str) -> Result<Vec<post::Model>, sea_orm::DbErr>;

  async fn find_all_by_text_order_by_id_asc(
    &self,
    text: &str,
  ) -> Result<Vec<post::Model>, sea_orm::DbErr>;

  async fn find_all_by_text_order_by_title_desc(
    &self,
    text: &str,
  ) -> Result<Vec<post::Model>, sea_orm::DbErr>;

  async fn find_all_by_text_limit(
    &self,
    text: &str,
    limit: u64,
  ) -> Result<Vec<post::Model>, sea_orm::DbErr>;

  async fn find_all_by_text_offset_limit(
    &self,
    text: &str,
    offset: u64,
    limit: u64,
  ) -> Result<Vec<post::Model>, sea_orm::DbErr>;

  async fn find_all_by_text_paginate(
    &self,
    text: &str,
    page: u64,
    per_page: u64,
  ) -> Result<Vec<post::Model>, sea_orm::DbErr>;

  async fn find_all_by_text_distinct(&self, text: &str)
  -> Result<Vec<post::Model>, sea_orm::DbErr>;

  async fn find_all_by_text_order_by_id_asc_limit(
    &self,
    text: &str,
    limit: u64,
  ) -> Result<Vec<post::Model>, sea_orm::DbErr>;

  async fn count_by_title(&self, title: &str) -> Result<u64, sea_orm::DbErr>;

  async fn delete_by_text(&self, text: &str) -> Result<sea_orm::DeleteResult, sea_orm::DbErr>;

  async fn exists_by_id(&self, id: i32) -> Result<bool, sea_orm::DbErr>;

  async fn update_text_by_id(
    &self,
    id: i32,
    text: String,
  ) -> Result<sea_orm::UpdateResult, sea_orm::DbErr>;

  async fn update_text_and_title_by_id(
    &self,
    id: i32,
    text: String,
    title: String,
  ) -> Result<sea_orm::UpdateResult, sea_orm::DbErr>;

  async fn find_by_id_gt(&self, id: i32) -> Result<Option<post::Model>, sea_orm::DbErr>;

  async fn find_all_by_text_like(&self, text: &str) -> Result<Vec<post::Model>, sea_orm::DbErr>;

  async fn find_by_id_gte_lte(
    &self,
    min_id: i32,
    max_id: i32,
  ) -> Result<Option<post::Model>, sea_orm::DbErr>;
}

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
