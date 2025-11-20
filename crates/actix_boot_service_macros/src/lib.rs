use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, FieldsNamed};

struct Field {
  ident: syn::Ident,
  ty: syn::Type,
}

const REPOSITORY_SUFFIX: &str = "_repository";

const SERVICE_SUFFIX: &str = "_service";

#[proc_macro_derive(Service)]
pub fn derive_service(input: TokenStream) -> TokenStream {
  impl_derive_service(parse_macro_input!(input as DeriveInput)).unwrap_or_else(|err| err.to_compile_error().into())
}

fn impl_derive_service(input: DeriveInput) -> syn::Result<TokenStream> {
  let ident = &input.ident;
  let fields = get_fields(&input)?.iter().map(|field| {
    let field_ident = &field.ident;
    let field_ty = &field.ty;
    let field_name = field_ident.to_string();

    if field_name.ends_with(REPOSITORY_SUFFIX) {
      return Ok(quote! {
        #field_ident: context.get_repository()
      });
    }

    if field_name.ends_with(SERVICE_SUFFIX) {
      return Ok(quote! {
        #field_ident: context.get_service()
      });
    }

    Err(syn::Error::new_spanned(
      field_ty,
      "Field must have _repository or _service suffix",
    ))
  }).collect::<syn::Result<Vec<_>>>()?;

  Ok(quote! {
    impl actix_boot::service::Service for #ident {
      fn new_service(context: &actix_boot::di::DIContext) -> Self {
        Self {
          #(#fields),*
        }
      }
    }
  }.into())
}

fn get_fields(input: &DeriveInput) -> syn::Result<Vec<Field>> {
  let Data::Struct(DataStruct { ref fields, .. }) = input.data else {
    return Err(syn::Error::new_spanned(
      input,
      "Use service only structs"
    ));
  };

  let Fields::Named(FieldsNamed { named, .. }) = fields else {
    return Err(syn::Error::new_spanned(
      input,
      "Use service only structs with named fields"
    ));
  };

  Ok(named.iter().map(|field| Field {
    ident: field.ident.clone().unwrap(),
    ty: field.ty.clone(),
  }).collect())
}