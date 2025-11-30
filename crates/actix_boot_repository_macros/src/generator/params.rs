use syn::TraitItemFn;
use crate::parse::ParseTokenCol;
use super::filter::FilterParam;

/// Extract parameters from function signature, skipping &self
pub fn extract_params(
  function: &TraitItemFn,
  columns: &[ParseTokenCol],
) -> syn::Result<Vec<FilterParam>> {
  let params: Vec<_> = function.sig.inputs.iter().skip(1).collect();

  // Count total filters needed
  let total_filters: usize = columns.iter()
    .map(|col| col.filters().len())
    .sum();

  if params.len() != total_filters {
    return Err(syn::Error::new_spanned(
      &function.sig.ident,
      format!(
        "Parameter count mismatch: expected {} parameters (one per filter), but found {}",
        total_filters,
        params.len()
      )
    ));
  }

  let mut result = Vec::new();
  let mut param_index = 0;

  for col in columns {
    let filter_count = col.filters().len();

    for _ in 0..filter_count {
      if param_index >= params.len() {
        return Err(syn::Error::new_spanned(
          &function.sig.ident,
          format!("Not enough parameters for filters on column '{}'", col.name)
        ));
      }

      let param = params[param_index];
      let ident = extract_param_ident(param)?;

      result.push(FilterParam {
        ident: ident.clone(),
        col: col.clone(),
      });

      param_index += 1;
    }
  }

  Ok(result)
}

/// Extract identifier from a function parameter
fn extract_param_ident(param: &syn::FnArg) -> syn::Result<syn::Ident> {
  match param {
    syn::FnArg::Typed(pat_type) => {
      match &*pat_type.pat {
        syn::Pat::Ident(ident) => Ok(ident.ident.clone()),
        _ => Err(syn::Error::new_spanned(
          &pat_type.pat,
          "Expected identifier pattern"
        ))
      }
    }
    _ => Err(syn::Error::new_spanned(
      param,
      "Expected typed parameter"
    ))
  }
}
