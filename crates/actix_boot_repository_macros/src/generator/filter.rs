use proc_macro2::Span;
use quote::quote;
use syn::Path;
use crate::parse::ParseTokenCol;

/// Converts snake_case to PascalCase for SeaORM Column enum variants
/// Example: "user_name" -> "UserName", "id" -> "Id"
pub fn to_pascal_case(s: &str) -> String {
  s.split('_')
    .map(|word| {
      let mut chars = word.chars();
      match chars.next() {
        None => String::new(),
        Some(first) => {
          first.to_uppercase().collect::<String>() + chars.as_str()
        }
      }
    })
    .collect()
}

/// Represents a parameter extracted from function signature
pub struct FilterParam {
  pub ident: syn::Ident,
  pub col: ParseTokenCol,
}

/// Generate filter chain for a column with its associated filters
pub fn generate_filter_chain(
  col: &ParseTokenCol,
  param_ident: &syn::Ident,
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  let pascal_name = to_pascal_case(&col.name);
  let col_name = syn::Ident::new(&pascal_name, Span::call_site());

  let filters = col.filters();

  if filters.is_empty() {
    return Ok(quote! {});
  }

  // For multiple filters on the same column, chain them
  let filter_methods = filters.iter().map(|filter| {
    filter.quote(param_ident)
  });

  Ok(quote! {
    .filter(#module::Column::#col_name #(#filter_methods)*)
  })
}

/// Generate all filters for the query
pub fn generate_all_filters(
  columns: &[ParseTokenCol],
  params: &[FilterParam],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  let mut filters = Vec::new();
  let mut param_index = 0;

  for col in columns {
    let pascal_name = to_pascal_case(&col.name);
    let col_name = syn::Ident::new(&pascal_name, proc_macro2::Span::call_site());
    let filter_list = col.filters();

    for filter in filter_list {
      if param_index >= params.len() {
        return Err(syn::Error::new(
          proc_macro2::Span::call_site(),
          format!(
            "Not enough parameters: column '{}' needs a parameter for filter but only {} parameters provided",
            col.name,
            params.len()
          )
        ));
      }

      let param_ident = &params[param_index].ident;
      let filter_method = filter.quote(param_ident);

      filters.push(quote! {
        .filter(#module::Column::#col_name #filter_method)
      });

      param_index += 1;
    }
  }

  Ok(quote! {
    #(#filters)*
  })
}
