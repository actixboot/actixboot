use quote::quote;
use syn::{Path, TraitItemFn};
use crate::parse::ParseToken;

mod filter;
mod query;
mod params;
mod modifiers;

use params::{extract_query_params, extract_update_params, extract_aggregate_params};
use query::{
  build_find_by_query, build_find_all_by_query, build_count_by_query,
  build_delete_by_query, build_exists_by_query, build_update_by_query,
  build_sum_by_query, build_avg_by_query, build_min_by_query, build_max_by_query,
};

pub fn generate_query(function: &TraitItemFn, module: &Path) -> syn::Result<proc_macro2::TokenStream> {
  let token = ParseToken::parse(&function.sig.ident.to_string())?;
  let fn_sig = &function.sig;

  let query_expr = match &token {
    ParseToken::FindBy(spec) => {
      let params = extract_query_params(function, spec)?;
      build_find_by_query(spec, &params, module)?
    }
    ParseToken::FindAllBy(spec) => {
      let params = extract_query_params(function, spec)?;
      build_find_all_by_query(spec, &params, module)?
    }
    ParseToken::CountBy(spec) => {
      let params = extract_query_params(function, spec)?;
      build_count_by_query(spec, &params, module)?
    }
    ParseToken::DeleteBy(spec) => {
      let params = extract_query_params(function, spec)?;
      build_delete_by_query(spec, &params, module)?
    }
    ParseToken::ExistsBy(spec) => {
      let params = extract_query_params(function, spec)?;
      build_exists_by_query(spec, &params, module)?
    }
    ParseToken::UpdateBy(spec) => {
      let params = extract_update_params(function, spec)?;
      build_update_by_query(spec, &params, module)?
    }
    ParseToken::SumBy(spec) => {
      let params = extract_aggregate_params(function, spec)?;
      build_sum_by_query(spec, &params, module)?
    }
    ParseToken::AvgBy(spec) => {
      let params = extract_aggregate_params(function, spec)?;
      build_avg_by_query(spec, &params, module)?
    }
    ParseToken::MinBy(spec) => {
      let params = extract_aggregate_params(function, spec)?;
      build_min_by_query(spec, &params, module)?
    }
    ParseToken::MaxBy(spec) => {
      let params = extract_aggregate_params(function, spec)?;
      build_max_by_query(spec, &params, module)?
    }
  };

  Ok(quote! {
    pub #fn_sig {
      #query_expr
    }
  })
}
