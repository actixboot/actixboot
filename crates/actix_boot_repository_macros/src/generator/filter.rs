use quote::quote;
use syn::Path;
use crate::parse::ParseTokenCol;

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

pub struct FilterParam {
  pub ident: syn::Ident,
}

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
