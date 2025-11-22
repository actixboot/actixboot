use std::str::FromStr;
use quote::quote;
use syn::{Path, TraitItemFn};
use crate::generator::find::generate_find_function;

pub mod find;

pub enum FnType {
  Find,
}

impl FromStr for FnType {
  type Err = syn::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "find" => Ok(Self::Find),
      _ => return Err(syn::Error::new(proc_macro2::Span::mixed_site(), "unrecognised syntax")),
    }
  }
}

pub fn generate_repository_function(item: &TraitItemFn, module: &Path) -> syn::Result<proc_macro2::TokenStream> {
  let fn_name = item.sig.ident.to_string();
  let fn_split = fn_name.split("_").collect::<Vec<_>>();
  let fn_type = FnType::from_str(fn_split.get(0).ok_or_else(|| syn::Error::new_spanned(item, "Missing fn type"))?)?;

  match fn_type {
    FnType::Find => generate_find_function(item, &module),
  }
}