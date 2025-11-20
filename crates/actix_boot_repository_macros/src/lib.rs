use proc_macro::TokenStream;
use proc_macro2::Span;
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
  let ident = &item.ident;
  let struct_name = ident.to_string().replace("Base", "");
  let struct_ident = syn::Ident::new(&struct_name, Span::call_site());
  let model = attr.model;
  let entity = attr.entity;

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
      type Model = #model;

      fn find_all(&self) -> impl std::future::Future<Output = std::result::Result<Vec<Self::Model>, sea_orm::DbErr>> {
        #entity::find().all(&self.db)
      }

      fn find(&self, id: i32) -> impl std::future::Future<Output = std::result::Result<Option<Self::Model>, sea_orm::DbErr>> {
        #entity::find_by_id(id).one(&self.db)
      }

      fn exists(&self, id: i32) -> impl std::future::Future<Output = std::result::Result<bool, sea_orm::DbErr>> + Send {
        async move {
          #entity::find_by_id(id)
            .count(&self.db)
            .await
            .map(|count| count > 0)
        }
      }
    }
  })
}
