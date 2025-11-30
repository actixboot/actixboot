use syn::TraitItemFn;
use crate::parse::{ParseTokenCol, QuerySpec, UpdateSpec, AggregateSpec};
use super::filter::FilterParam;
use super::modifiers::count_modifier_params;

pub fn extract_params(
  function: &TraitItemFn,
  columns: &[ParseTokenCol],
) -> syn::Result<Vec<FilterParam>> {
  let params: Vec<_> = function.sig.inputs.iter().skip(1).collect();

  let total_filters: usize = columns.iter()
    .map(|col| col.filters().iter().filter(|f| f.needs_param()).count())
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
    for filter in col.filters() {
      if !filter.needs_param() {
        continue;
      }

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
      });

      param_index += 1;
    }
  }

  Ok(result)
}

pub fn extract_query_params(
  function: &TraitItemFn,
  spec: &QuerySpec,
) -> syn::Result<Vec<FilterParam>> {
  let params: Vec<_> = function.sig.inputs.iter().skip(1).collect();

  let filter_count: usize = spec.filters.iter()
    .map(|col| col.filters().iter().filter(|f| f.needs_param()).count())
    .sum();

  let modifier_count = count_modifier_params(&spec.modifiers);
  let total_expected = filter_count + modifier_count;

  if params.len() != total_expected {
    return Err(syn::Error::new_spanned(
      &function.sig.ident,
      format!(
        "Parameter count mismatch: expected {} parameters ({} filters + {} modifiers), but found {}",
        total_expected,
        filter_count,
        modifier_count,
        params.len()
      )
    ));
  }

  let mut result = Vec::new();
  let mut param_index = 0;

  for col in &spec.filters {
    for filter in col.filters() {
      if !filter.needs_param() {
        continue;
      }

      let param = params[param_index];
      let ident = extract_param_ident(param)?;

      result.push(FilterParam {
        ident: ident.clone(),
      });

      param_index += 1;
    }
  }

  Ok(result)
}

pub fn extract_update_params(
  function: &TraitItemFn,
  spec: &UpdateSpec,
) -> syn::Result<Vec<FilterParam>> {
  let params: Vec<_> = function.sig.inputs.iter().skip(1).collect();

  let update_count = spec.updates.len();
  let filter_count: usize = spec.filters.iter()
    .map(|col| col.filters().iter().filter(|f| f.needs_param()).count())
    .sum();

  let modifier_count = count_modifier_params(&spec.modifiers);
  let total_expected = update_count + filter_count + modifier_count;

  if params.len() != total_expected {
    return Err(syn::Error::new_spanned(
      &function.sig.ident,
      format!(
        "Parameter count mismatch: expected {} parameters ({} updates + {} filters + {} modifiers), but found {}",
        total_expected,
        update_count,
        filter_count,
        modifier_count,
        params.len()
      )
    ));
  }

  let mut result = Vec::new();

  for (i, _col_name) in spec.updates.iter().enumerate() {
    let param = params[i];
    let ident = extract_param_ident(param)?;

    result.push(FilterParam {
      ident: ident.clone(),
    });
  }

  let mut param_index = update_count;

  for col in &spec.filters {
    for filter in col.filters() {
      if !filter.needs_param() {
        continue;
      }

      let param = params[param_index];
      let ident = extract_param_ident(param)?;

      result.push(FilterParam {
        ident: ident.clone(),
      });

      param_index += 1;
    }
  }

  Ok(result)
}

pub fn extract_aggregate_params(
  function: &TraitItemFn,
  spec: &AggregateSpec,
) -> syn::Result<Vec<FilterParam>> {
  extract_params(function, &spec.filters)
}

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
