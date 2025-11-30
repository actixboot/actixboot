use quote::quote;
use syn::Path;
use crate::parse::{QuerySpec, UpdateSpec, AggregateSpec, QueryModifier};
use super::filter::{FilterParam, generate_all_filters};
use super::modifiers::generate_modifiers;

pub fn build_find_by_query(
  spec: &QuerySpec,
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  let filters = generate_all_filters(&spec.filters, params, module)?;
  let modifiers = generate_modifiers(&spec.modifiers, module)?;

  Ok(quote! {
    {
      use sea_orm::{QueryOrder, QuerySelect};
      #module::Entity::find()
        #filters
        #modifiers
        .one(&self.db)
        .await
    }
  })
}

pub fn build_find_all_by_query(
  spec: &QuerySpec,
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  let has_paginate = spec.modifiers.iter().any(|m| matches!(m, QueryModifier::Paginate));

  if has_paginate {
    let filters = generate_all_filters(&spec.filters, params, module)?;

    Ok(quote! {
      {
        use sea_orm::{QueryOrder, QuerySelect};
        use actix_boot::repository::Paginator;

        let total_items = #module::Entity::find()
          #filters
          .count(&self.db)
          .await?;

        let total_pages = if per_page > 0 {
          (total_items + per_page - 1) / per_page
        } else {
          0
        };

        let items = #module::Entity::find()
          #filters
          .offset((page.saturating_sub(1)) * per_page)
          .limit(per_page)
          .all(&self.db)
          .await?;

        Ok(Paginator {
          items,
          page,
          per_page,
          total_items,
          total_pages,
        })
      }
    })
  } else {
    let filters = generate_all_filters(&spec.filters, params, module)?;
    let modifiers = generate_modifiers(&spec.modifiers, module)?;

    Ok(quote! {
      {
        use sea_orm::{QueryOrder, QuerySelect};
        #module::Entity::find()
          #filters
          #modifiers
          .all(&self.db)
          .await
      }
    })
  }
}

pub fn build_count_by_query(
  spec: &QuerySpec,
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  let filters = generate_all_filters(&spec.filters, params, module)?;
  let modifiers = generate_modifiers(&spec.modifiers, module)?;

  Ok(quote! {
    {
      use sea_orm::{QueryOrder, QuerySelect};
      #module::Entity::find()
        #filters
        #modifiers
        .count(&self.db)
        .await
    }
  })
}

pub fn build_delete_by_query(
  spec: &QuerySpec,
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  let filters = generate_all_filters(&spec.filters, params, module)?;

  Ok(quote! {
    #module::Entity::delete_many()
      #filters
      .exec(&self.db)
      .await
  })
}

pub fn build_exists_by_query(
  spec: &QuerySpec,
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  let filters = generate_all_filters(&spec.filters, params, module)?;

  Ok(quote! {
    {
      let count = #module::Entity::find()
        #filters
        .count(&self.db)
        .await?;
      Ok(count > 0)
    }
  })
}

pub fn build_update_by_query(
  spec: &UpdateSpec,
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  use super::filter::to_pascal_case;

  let update_count = spec.updates.len();
  let filter_params = &params[update_count..];
  let update_params = &params[..update_count];

  let updates: Vec<_> = spec.updates.iter().zip(update_params.iter()).map(|(col_name, param)| {
    let pascal_name = to_pascal_case(col_name);
    let col_ident = syn::Ident::new(&pascal_name, proc_macro2::Span::call_site());
    let param_ident = &param.ident;

    quote! {
      .col_expr(#module::Column::#col_ident, sea_orm::sea_query::Expr::value(#param_ident))
    }
  }).collect();

  let filters = generate_all_filters(&spec.filters, filter_params, module)?;

  Ok(quote! {
    #module::Entity::update_many()
      #(#updates)*
      #filters
      .exec(&self.db)
      .await
  })
}

pub fn build_sum_by_query(
  spec: &AggregateSpec,
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  use super::filter::to_pascal_case;

  let pascal_name = to_pascal_case(&spec.column);
  let col_ident = syn::Ident::new(&pascal_name, proc_macro2::Span::call_site());
  let filters = generate_all_filters(&spec.filters, params, module)?;

  Ok(quote! {
    {
      use sea_orm::sea_query::Expr;

      #module::Entity::find()
        #filters
        .select_only()
        .column_as(#module::Column::#col_ident.sum(), "sum")
        .into_tuple()
        .one(&self.db)
        .await
    }
  })
}

pub fn build_avg_by_query(
  spec: &AggregateSpec,
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  use super::filter::to_pascal_case;

  let pascal_name = to_pascal_case(&spec.column);
  let col_ident = syn::Ident::new(&pascal_name, proc_macro2::Span::call_site());
  let filters = generate_all_filters(&spec.filters, params, module)?;

  Ok(quote! {
    {
      use sea_orm::sea_query::Expr;

      #module::Entity::find()
        #filters
        .select_only()
        .column_as(#module::Column::#col_ident.avg(), "avg")
        .into_tuple()
        .one(&self.db)
        .await
    }
  })
}

pub fn build_min_by_query(
  spec: &AggregateSpec,
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  use super::filter::to_pascal_case;

  let pascal_name = to_pascal_case(&spec.column);
  let col_ident = syn::Ident::new(&pascal_name, proc_macro2::Span::call_site());
  let filters = generate_all_filters(&spec.filters, params, module)?;

  Ok(quote! {
    {
      use sea_orm::sea_query::Expr;

      #module::Entity::find()
        #filters
        .select_only()
        .column_as(#module::Column::#col_ident.min(), "min")
        .into_tuple()
        .one(&self.db)
        .await
    }
  })
}

pub fn build_max_by_query(
  spec: &AggregateSpec,
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  use super::filter::to_pascal_case;

  let pascal_name = to_pascal_case(&spec.column);
  let col_ident = syn::Ident::new(&pascal_name, proc_macro2::Span::call_site());
  let filters = generate_all_filters(&spec.filters, params, module)?;

  Ok(quote! {
    {
      use sea_orm::sea_query::Expr;

      #module::Entity::find()
        #filters
        .select_only()
        .column_as(#module::Column::#col_ident.max(), "max")
        .into_tuple()
        .one(&self.db)
        .await
    }
  })
}
