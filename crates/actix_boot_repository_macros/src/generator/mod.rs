use quote::quote;
use syn::{Path, TraitItemFn};
use crate::parse::ParseToken;

mod filter;
mod query;
mod params;

use params::extract_params;
use query::{build_find_by_query, build_find_all_by_query, build_count_by_query, build_delete_by_query};

/// Generate the complete implementation for a repository function
pub fn generate_query(function: &TraitItemFn, module: &Path) -> syn::Result<proc_macro2::TokenStream> {
  let token = ParseToken::parse(&function.sig.ident.to_string())?;
  let fn_name = &function.sig.ident;
  let fn_sig = &function.sig;

  let (columns, query_expr) = match &token {
    ParseToken::FindBy(columns) => {
      let params = extract_params(function, columns)?;
      let query = build_find_by_query(columns, &params, module)?;
      (columns, query)
    }
    ParseToken::FindAllBy(columns) => {
      let params = extract_params(function, columns)?;
      let query = build_find_all_by_query(columns, &params, module)?;
      (columns, query)
    }
    ParseToken::CountBy(columns) => {
      let params = extract_params(function, columns)?;
      let query = build_count_by_query(columns, &params, module)?;
      (columns, query)
    }
    ParseToken::DeleteBy(columns) => {
      let params = extract_params(function, columns)?;
      let query = build_delete_by_query(columns, &params, module)?;
      (columns, query)
    }
  };

  // Validate that we have at least one column
  if columns.is_empty() {
    return Err(syn::Error::new_spanned(
      fn_name,
      "Repository function must specify at least one column"
    ));
  }

  Ok(quote! {
    pub #fn_sig {
      #query_expr
    }
  })
}
