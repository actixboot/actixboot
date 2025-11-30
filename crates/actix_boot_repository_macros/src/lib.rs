use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{TraitItem, parse_macro_input, ItemTrait, Path};
use crate::generator::generate_query;

mod generator;
mod parse;

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
  module: Path,
}

impl Parse for RepositoryAttr {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let module: Path = input.parse()?;
    Ok(RepositoryAttr { module })
  }
}

fn impl_repository(attr: RepositoryAttr, item: ItemTrait) -> syn::Result<proc_macro2::TokenStream> {
  let ident = &item.ident;
  let struct_name = ident.to_string().replace("Base", "");
  let struct_ident = syn::Ident::new(&struct_name, Span::call_site());
  let module = attr.module;

  let functions = item.items.iter()
    .filter_map(|item| match item {
      TraitItem::Fn(function) => Some(function),
      _ => return None,
    })
    .map(|function| generate_query(function, &module))
    .collect::<syn::Result<Vec<_>>>()?;

  Ok(quote! {
    struct #struct_ident {
      db: sea_orm::DatabaseConnection,
    }

    impl From<sea_orm::DatabaseConnection> for #struct_ident {
      fn from(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
      }
    }

    impl actix_boot::di::GetOrCreate for #struct_ident {
      fn get_or_create(ctx: &actix_boot::di::DIContext) -> actix_web::web::Data<Self> {
        actix_web::web::Data::from(ctx.get_repository::<#struct_ident>())
      }
    }

    impl actix_boot::repository::Repository for #struct_ident {
      type Model = #module::Model;

      fn find_all(&self) -> impl std::future::Future<Output = std::result::Result<Vec<Self::Model>, sea_orm::DbErr>> {
        #module::Entity::find().all(&self.db)
      }

      fn find(&self, id: i32) -> impl std::future::Future<Output = std::result::Result<Option<Self::Model>, sea_orm::DbErr>> {
        #module::Entity::find_by_id(id).one(&self.db)
      }

      fn exists(&self, id: i32) -> impl std::future::Future<Output = std::result::Result<bool, sea_orm::DbErr>> + Send {
        async move {
          #module::Entity::find_by_id(id)
            .count(&self.db)
            .await
            .map(|count| count > 0)
        }
      }
    }

    impl #struct_ident {
      #(#functions)*
    }
  })
}
