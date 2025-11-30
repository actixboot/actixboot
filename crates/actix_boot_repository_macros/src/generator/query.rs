use quote::quote;
use syn::Path;
use crate::parse::ParseTokenCol;
use super::filter::{FilterParam, generate_all_filters};

/// Build a find_by query (returns Option<Model>)
pub fn build_find_by_query(
  columns: &[ParseTokenCol],
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  let filters = generate_all_filters(columns, params, module)?;

  Ok(quote! {
    #module::Entity::find()
      #filters
      .one(&self.db)
      .await
  })
}

/// Build a find_all_by query (returns Vec<Model>)
pub fn build_find_all_by_query(
  columns: &[ParseTokenCol],
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  let filters = generate_all_filters(columns, params, module)?;

  Ok(quote! {
    #module::Entity::find()
      #filters
      .all(&self.db)
      .await
  })
}

/// Build a count_by query (returns u64)
pub fn build_count_by_query(
  columns: &[ParseTokenCol],
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  let filters = generate_all_filters(columns, params, module)?;

  Ok(quote! {
    #module::Entity::find()
      #filters
      .count(&self.db)
      .await
  })
}

/// Build a delete_by query (returns DeleteResult)
pub fn build_delete_by_query(
  columns: &[ParseTokenCol],
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  let filters = generate_all_filters(columns, params, module)?;

  Ok(quote! {
    #module::Entity::delete_many()
      #filters
      .exec(&self.db)
      .await
  })
}
