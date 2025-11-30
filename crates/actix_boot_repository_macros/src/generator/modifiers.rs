use quote::quote;
use syn::Path;
use crate::parse::{QueryModifier, OrderDirection};
use super::filter::to_pascal_case;

pub fn generate_modifiers(
  modifiers: &[QueryModifier],
  module: &Path,
) -> syn::Result<proc_macro2::TokenStream> {
  use quote::quote;
  let mut result = Vec::new();

  for modifier in modifiers {
    match modifier {
      QueryModifier::OrderBy(col_name, direction) => {
        let pascal_name = to_pascal_case(col_name);
        let col_ident = syn::Ident::new(&pascal_name, proc_macro2::Span::call_site());

        let order_type = match direction {
          OrderDirection::Asc => quote! { sea_orm::Order::Asc },
          OrderDirection::Desc => quote! { sea_orm::Order::Desc },
        };

        result.push(quote! {
          .order_by(#module::Column::#col_ident, #order_type)
        });
      },
      QueryModifier::Distinct => {
        result.push(quote! {
          .distinct()
        });
      },
      QueryModifier::Limit => {
        result.push(quote! {
          .limit(limit)
        });
      },
      QueryModifier::Offset => {
        result.push(quote! {
          .offset(offset)
        });
      },
      QueryModifier::Paginate => {
        result.push(quote! {
          .offset((page.saturating_sub(1)) * per_page)
          .limit(per_page)
        });
      },
    }
  }

  Ok(quote! {
    #(#result)*
  })
}

pub fn count_modifier_params(modifiers: &[QueryModifier]) -> usize {
  modifiers.iter().map(|m| match m {
    QueryModifier::Limit => 1,
    QueryModifier::Offset => 1,
    QueryModifier::Paginate => 2,
    QueryModifier::OrderBy(_, _) => 0,
    QueryModifier::Distinct => 0,
  }).sum()
}
