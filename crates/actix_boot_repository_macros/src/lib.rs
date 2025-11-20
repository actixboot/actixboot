use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{ItemFn, TraitItem, Type, parse_macro_input, Meta, Token, MetaNameValue, ItemTrait};
use syn::punctuated::Punctuated;

#[proc_macro_attribute]
pub fn repository(attr: TokenStream, item: TokenStream) -> TokenStream {
  impl_repository(
    parse_macro_input!(attr as RepositoryAttr),
    parse_macro_input!(item as ItemTrait),
  )
  .unwrap_or_else(|err| err.to_compile_error().into())
  .into()
}

struct RepositoryAttr {
  entity: Type,
  model: Type,
}

impl Parse for RepositoryAttr {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let mut entity = None;
    let mut model = None;

    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let metas = parser(input)?;

    for meta in metas {
      if let Meta::NameValue(MetaNameValue { path, value, .. }) = meta {
        let key = path.get_ident()
          .ok_or_else(|| syn::Error::new_spanned(&path, "Expected identifier"))?
          .to_string();

        if let syn::Expr::Path(expr_path) = value {
          let ty = Type::Path(syn::TypePath {
            qself: None,
            path: expr_path.path,
          });

          match key.as_str() {
            "entity" => entity = Some(ty),
            "model" => model = Some(ty),
            _ => return Err(syn::Error::new_spanned(path, "Unknown attribute")),
          }
        } else {
          return Err(syn::Error::new_spanned(value, "Expected a type path"));
        }
      }
    }

    Ok(RepositoryAttr {
      entity: entity.ok_or_else(|| input.error("Missing 'entity'"))?,
      model: model.ok_or_else(|| input.error("Missing 'model'"))?,
    })
  }
}

fn impl_repository(attr: RepositoryAttr, item: ItemTrait) -> syn::Result<proc_macro2::TokenStream> {
  Ok(quote! {#item})
}
