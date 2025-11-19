use proc_macro::{TokenStream};
use proc_macro2::Ident;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{ItemTrait, TraitItem, Type, parse_macro_input, FnArg, Pat, ReturnType};

struct RepositoryArgs {
  entity: Type,
}

impl Parse for RepositoryArgs {
  fn parse(input: ParseStream) -> syn::Result<Self> {
    let entity: Type = input.parse()?;

    Ok(Self { entity })
  }
}

struct Function {
  name: Ident,
  return_ty: ReturnType,
  args: Vec<FunctionArg>,
}

struct FunctionArg {
  name: Box<Pat>,
  ty: Box<Type>,
}

#[proc_macro_attribute]
pub fn repository(attr: TokenStream, item: TokenStream) -> TokenStream {
  let item_trait = parse_macro_input!(item as ItemTrait);
  let args = parse_macro_input!(attr as RepositoryArgs);
  let entity = &args.entity;
  let trait_name = &item_trait.ident;
  let mut functions = Vec::new();

  for item in &item_trait.items {
    let TraitItem::Fn(function) = item else {
      continue;
    };

    let fn_name = &function.sig.ident;
    let return_type = &function.sig.output;
    let mut args = Vec::new();
    let mut has_self = false;

    for arg in &function.sig.inputs {
      match arg {
        FnArg::Receiver(_) => has_self = true,
        FnArg::Typed(arg) => args.push(FunctionArg {
          name: arg.pat.clone(),
          ty: arg.ty.clone(),
        }),
      }
    }

    if !has_self {
      continue;
    }

    functions.push(Function {
      name: fn_name.clone(),
      return_ty: return_type.clone(),
      args,
    });
  }

  let quote_functions = functions.iter().map(|function| {
    let name = &function.name;
    let return_ty = &function.return_ty;
    let arg_names = function.args.iter().map(|arg| &arg.name).collect::<Vec<_>>();
    let arg_types = function.args.iter().map(|arg| &arg.ty).collect::<Vec<_>>();

    quote! {
      pub async fn #name(&self, #(#arg_names: #arg_types),*) #return_ty {
        #entity::find_by_id(2).one(&self.db).await
      }
    }
  }).collect::<Vec<_>>();

  quote! {
    pub struct #trait_name {
      db: sea_orm::prelude::DatabaseConnection,
    }

    impl #trait_name {
      #(#quote_functions)*
    }

    impl Repository for #trait_name {
        fn with_db(db: DatabaseConnection) -> Self {
          Self {
            db
          }
        }
      }
  }
  .into()
}
