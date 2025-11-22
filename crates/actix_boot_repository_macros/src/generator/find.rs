use quote::quote;
use std::str::FromStr;
use proc_macro2::Span;
use syn::{Path, TraitItemFn, FnArg, ReturnType};

pub enum Part {
  Find,
  By,
  Col(syn::Ident),
  And,
}

fn capitalize_first_letter(s: &str) -> String {
  let mut chars = s.chars();
  match chars.next() {
    None => String::new(),
    Some(first) => first.to_uppercase().chain(chars).collect(),
  }
}

impl FromStr for Part {
  type Err = syn::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "by" => Ok(Part::By),
      "and" => Ok(Part::And),
      "find" => Ok(Part::Find),
      s => Ok(Part::Col(syn::Ident::new(
        &capitalize_first_letter(s),
        proc_macro2::Span::call_site()
      ))),
    }
  }
}

pub fn generate_find_function(
  item: &TraitItemFn,
  module: &Path
) -> syn::Result<proc_macro2::TokenStream> {
  let fn_name = &item.sig.ident;
  let fn_name_str = fn_name.to_string();

  if !fn_name_str.starts_with("find") {
    return Err(syn::Error::new_spanned(
      fn_name,
      "Method name must start with 'find'"
    ));
  }

  let parts: Vec<Part> = fn_name_str
    .split('_')
    .map(Part::from_str)
    .collect::<Result<Vec<_>, _>>()?;

  validate_method_structure(&parts, item)?;

  let columns = extract_columns(&parts)?;
  let params = extract_params(item)?;

  if columns.len() != params.len() {
    return Err(syn::Error::new_spanned(
      item,
      format!(
        "Method has {} columns but {} parameters. They must match!",
        columns.len(),
        params.len()
      )
    ));
  }

  let query_chain = generate_query_chain(module, &columns, &params);
  let sig = &item.sig;
  let return_type = &sig.output;

  Ok(quote! {
    #sig {
      #query_chain
        .one(&self.db)
        .await
    }
  })
}

fn validate_method_structure(parts: &[Part], item: &TraitItemFn) -> syn::Result<()> {
  if !matches!(parts.first(), Some(Part::Find)) {
    return Err(syn::Error::new_spanned(
      item,
      "Method name must start with 'find'"
    ));
  }

  let has_by = parts.iter().any(|p| matches!(p, Part::By));
  if !has_by {
    return Err(syn::Error::new_spanned(
      item,
      "Method name must contain 'by' (e.g., find_by_name)"
    ));
  }

  let mut expecting_col = false;
  let mut found_by = false;

  for (i, part) in parts.iter().enumerate() {
    match part {
      Part::Find => {
        if i != 0 {
          return Err(syn::Error::new_spanned(
            item,
            "'find' must be at the start"
          ));
        }
      }
      Part::By => {
        if found_by {
          return Err(syn::Error::new_spanned(
            item,
            "Only one 'by' is allowed"
          ));
        }
        found_by = true;
        expecting_col = true;
      }
      Part::Col(_) => {
        if !found_by {
          return Err(syn::Error::new_spanned(
            item,
            "Column names must come after 'by'"
          ));
        }
        expecting_col = false;
      }
      Part::And => {
        if !found_by {
          return Err(syn::Error::new_spanned(
            item,
            "'and' must come after 'by'"
          ));
        }
        if expecting_col {
          return Err(syn::Error::new_spanned(
            item,
            "'and' must be followed by a column name"
          ));
        }
        expecting_col = true;
      }
    }
  }

  if expecting_col {
    return Err(syn::Error::new_spanned(
      item,
      "Method name must end with a column name"
    ));
  }

  Ok(())
}

fn extract_columns(parts: &[Part]) -> syn::Result<Vec<syn::Ident>> {
  let mut columns = Vec::new();

  for part in parts {
    if let Part::Col(name) = part {
      columns.push(name.clone());
    }
  }

  Ok(columns)
}

fn extract_params(item: &TraitItemFn) -> syn::Result<Vec<syn::Ident>> {
  let mut params = Vec::new();

  for arg in &item.sig.inputs {
    match arg {
      FnArg::Receiver(_) => continue,
      FnArg::Typed(pat_type) => {
        if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
          params.push(pat_ident.ident.clone());
        } else {
          return Err(syn::Error::new_spanned(
            pat_type,
            "Only simple parameter names are supported"
          ));
        }
      }
    }
  }

  Ok(params)
}

fn generate_query_chain(
  module: &Path,
  columns: &[syn::Ident],
  params: &[syn::Ident]
) -> proc_macro2::TokenStream {
  let mut filters = Vec::new();

  for (col, param) in columns.iter().zip(params.iter()) {
    if filters.is_empty() {
      filters.push(quote! {
        .filter(#module::Column::#col.eq(#param))
      });
    } else {
      filters.push(quote! {
        .filter(#module::Column::#col.eq(#param))
      });
    }
  }

  quote! {
    #module::Entity::find()
      #(#filters)*
  }
}