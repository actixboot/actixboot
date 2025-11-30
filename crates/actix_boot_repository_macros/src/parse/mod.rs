use std::iter::Peekable;
use std::slice::Iter;
use quote::quote;
use crate::parse::FnType::Find;

#[derive(Debug)]
pub enum ParseToken {
  FindBy(Vec<ParseTokenCol>),
  FindAllBy(Vec<ParseTokenCol>),
  CountBy(Vec<ParseTokenCol>),
  DeleteBy(Vec<ParseTokenCol>),
}

#[derive(Debug, Clone)]
pub struct ParseTokenCol {
  pub name: String,
  filters: Vec<ColFilter>
}

impl ParseTokenCol {
  pub fn filters(&self) -> Vec<ColFilter> {
    if self.filters.len() == 0 {
      vec![ColFilter::Eq]
    } else {
      self.filters.clone()
    }
  }
}

enum FnType {
  Find,
  FindAll,
  Count,
  Delete
}

impl FnType {
  pub fn into_parse_token(self, tokens: Vec<ParseTokenCol>) -> ParseToken {
    match self {
      Find => ParseToken::FindBy(tokens),
      FnType::FindAll => ParseToken::FindAllBy(tokens),
      FnType::Count => ParseToken::CountBy(tokens),
      FnType::Delete => ParseToken::DeleteBy(tokens),
    }
  }

  fn parse(tokens: &mut Peekable<Iter<&str>>, function_name: &str) -> syn::Result<Self> {
    let token = *tokens.next().ok_or_else(|| Self::err(function_name))?;

    match token {
      "find" => {
        let has_all = tokens.peek().map(|token| token.eq(&&"all")).unwrap_or(false);
        if has_all {
          tokens.next();
          Ok(Self::FindAll)
        } else {
          Ok(Self::Find)
        }
      },
      "count" => Ok(Self::Count),
      "delete" => Ok(Self::Delete),
      _ => Err(Self::err(token)),
    }
  }

  fn err(token: &str) -> syn::Error {
    syn::Error::new_spanned(token, "Unknow token. Expecting find, find_all, count or delete")
  }
}

impl ParseToken {
  pub fn parse(function_name: &str) -> syn::Result<Self> {
    let split = function_name.split("_").collect::<Vec<_>>();
    let mut tokens = vec![];
    let mut iter = split.iter().peekable();
    let fn_type = FnType::parse(&mut iter, function_name)?;

    if iter.peek() == Some(&&"by") {
      iter.next();
    } else {
      return Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "Expected 'by' after function type (e.g., 'find_by_', 'find_all_by_')"
      ));
    }

    loop {
      let part = match iter.next() {
        Some(p) => p,
        None => break,
      };

      if *part == "and" {
        return Err(syn::Error::new(
          proc_macro2::Span::call_site(),
          "Unexpected 'and' - expected column name"
        ));
      }

      let mut col_name_parts = vec![*part];
      let mut filters = vec![];

      let mut saw_and = false;
      loop {
        if let Some(filter) = ColFilter::parse(&mut iter)? {
          filters.push(filter);
          continue;
        }

        match iter.peek() {
          Some(&&"and") => {
            iter.next();
            saw_and = true;
            break;
          }
          Some(next_part) => {
            col_name_parts.push(*next_part);
            iter.next();
          }
          None => {
            break;
          }
        }
      }

      tokens.push(ParseTokenCol {
        name: col_name_parts.join("_"),
        filters,
      });

      if saw_and && iter.peek().is_none() {
        return Err(syn::Error::new(
          proc_macro2::Span::call_site(),
          "Expected column name after 'and', but found end of function name"
        ));
      }

      if iter.peek().is_some() {
        continue;
      } else {
        break;
      }
    }

    Ok(fn_type.into_parse_token(tokens))
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColFilter {
  Eq,
  NotEq,
  Gt,
  Gte,
  Lt,
  Lte,
  NotBetween,
  Between,
  EndsWith,
  StartsWith,
  Contains,
  Like,
  NotLike,
  IsIn,
  IsNotIn,
  IsNull,
  IsNotNull,
}

impl ColFilter {
  pub fn quote(&self, param_ident: &syn::Ident) -> proc_macro2::TokenStream {
    match self {
      ColFilter::Eq => quote! {.eq(#param_ident)},
      ColFilter::NotEq => quote! {.ne(#param_ident)},
      ColFilter::Gt => quote! {.gt(#param_ident)},
      ColFilter::Gte => quote! {.gte(#param_ident)},
      ColFilter::Lt => quote! {.lt(#param_ident)},
      ColFilter::Lte => quote! {.lte(#param_ident)},
      ColFilter::Between => quote! {.between(#param_ident)},
      ColFilter::NotBetween => quote! {.not_between(#param_ident)},
      ColFilter::EndsWith => quote! {.ends_with(#param_ident)},
      ColFilter::StartsWith => quote! {.starts_with(#param_ident)},
      ColFilter::Contains => quote! {.contains(#param_ident)},
      ColFilter::Like => quote! {.like(#param_ident)},
      ColFilter::NotLike => quote! {.not_like(#param_ident)},
      ColFilter::IsIn => quote! {.is_in(#param_ident)},
      ColFilter::IsNotIn => quote! {.is_not_in(#param_ident)},
      ColFilter::IsNull => quote! {.is_null()},
      ColFilter::IsNotNull => quote! {.is_not_null()},
    }
  }
}

impl ColFilter {
  pub fn parse(iter: &mut Peekable<Iter<&str>>) -> syn::Result<Option<ColFilter>> {
    let first = match iter.peek() {
      Some(token) => **token,
      None => return Ok(None),
    };

    let result = match first {
      "eq" => {
        iter.next();
        Some(ColFilter::Eq)
      },
      "gt" => {
        iter.next();
        Some(ColFilter::Gt)
      },
      "gte" => {
        iter.next();
        Some(ColFilter::Gte)
      },
      "lt" => {
        iter.next();
        Some(ColFilter::Lt)
      },
      "lte" => {
        iter.next();
        Some(ColFilter::Lte)
      },
      "between" => {
        iter.next();
        Some(ColFilter::Between)
      },
      "contains" => {
        iter.next();
        Some(ColFilter::Contains)
      },
      "like" => {
        iter.next();
        Some(ColFilter::Like)
      },

      "not" => {
        iter.next();
        match iter.peek() {
          Some(&&"eq") => {
            iter.next();
            Some(ColFilter::NotEq)
          },
          Some(&&"between") => {
            iter.next();
            Some(ColFilter::NotBetween)
          },
          Some(&&"like") => {
            iter.next();
            Some(ColFilter::NotLike)
          },
          Some(&&next) => {
            return Err(syn::Error::new(
              proc_macro2::Span::call_site(),
              format!("Invalid filter: 'not' must be followed by 'eq', 'between', or 'like', but found '{}'", next)
            ));
          },
          None => {
            return Err(syn::Error::new(
              proc_macro2::Span::call_site(),
              "Invalid filter: 'not' must be followed by 'eq', 'between', or 'like'"
            ));
          }
        }
      },

      "starts" => {
        iter.next();
        match iter.peek() {
          Some(&&"with") => {
            iter.next();
            Some(ColFilter::StartsWith)
          },
          Some(&&next) => {
            return Err(syn::Error::new(
              proc_macro2::Span::call_site(),
              format!("Invalid filter: 'starts' must be followed by 'with', but found '{}'", next)
            ));
          },
          None => {
            return Err(syn::Error::new(
              proc_macro2::Span::call_site(),
              "Invalid filter: 'starts' must be followed by 'with'"
            ));
          }
        }
      },

      "ends" => {
        iter.next();
        match iter.peek() {
          Some(&&"with") => {
            iter.next();
            Some(ColFilter::EndsWith)
          },
          Some(&&next) => {
            return Err(syn::Error::new(
              proc_macro2::Span::call_site(),
              format!("Invalid filter: 'ends' must be followed by 'with', but found '{}'", next)
            ));
          },
          None => {
            return Err(syn::Error::new(
              proc_macro2::Span::call_site(),
              "Invalid filter: 'ends' must be followed by 'with'"
            ));
          }
        }
      },

      "is" => {
        iter.next();
        match iter.peek() {
          Some(&&"null") => {
            iter.next();
            Some(ColFilter::IsNull)
          },
          Some(&&"not") => {
            iter.next();
            match iter.peek() {
              Some(&&"null") => {
                iter.next();
                Some(ColFilter::IsNotNull)
              },
              Some(&&"in") => {
                iter.next();
                Some(ColFilter::IsNotIn)
              },
              Some(&&next) => {
                return Err(syn::Error::new(
                  proc_macro2::Span::call_site(),
                  format!("Invalid filter: 'is_not' must be followed by 'null' or 'in', but found '{}'", next)
                ));
              },
              None => {
                return Err(syn::Error::new(
                  proc_macro2::Span::call_site(),
                  "Invalid filter: 'is_not' must be followed by 'null' or 'in'"
                ));
              }
            }
          },
          Some(&&"in") => {
            iter.next();
            Some(ColFilter::IsIn)
          },
          Some(&&next) => {
            return Err(syn::Error::new(
              proc_macro2::Span::call_site(),
              format!("Invalid filter: 'is' must be followed by 'null', 'not', or 'in', but found '{}'", next)
            ));
          },
          None => {
            return Err(syn::Error::new(
              proc_macro2::Span::call_site(),
              "Invalid filter: 'is' must be followed by 'null', 'not', or 'in'"
            ));
          }
        }
      },

      _ => None,
    };

    Ok(result)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_find_by_with_filter() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_text_not_eq").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "text");
    assert_eq!(cols[0].filters, vec![ColFilter::NotEq]);
  }

  #[test]
  fn test_find_by_single_column() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_id").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "id");
    assert_eq!(cols[0].filters(), vec![ColFilter::Eq]);
  }

  #[test]
  fn test_find_by_multiple_columns() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_name_and_age").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].name, "name");
    assert_eq!(cols[1].name, "age");
  }

  #[test]
  fn test_count_by() {
    let ParseToken::CountBy(cols) = ParseToken::parse("count_by_category").unwrap() else {
      panic!("expected CountBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "category");
  }

  #[test]
  fn test_delete_by() {
    let ParseToken::DeleteBy(cols) = ParseToken::parse("delete_by_id").unwrap() else {
      panic!("expected DeleteBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "id");
  }

  #[test]
  fn test_filter_eq() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_name_eq").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::Eq]);
  }

  #[test]
  fn test_filter_gt() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_age_gt").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::Gt]);
  }

  #[test]
  fn test_filter_gte() {
    let result = ParseToken::parse("find_by_age_gte").unwrap();
    let ParseToken::FindBy(cols) = result else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::Gte]);
  }

  #[test]
  fn test_filter_lt() {
    let result = ParseToken::parse("find_by_price_lt").unwrap();
    let ParseToken::FindBy(cols) = result else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::Lt]);
  }

  #[test]
  fn test_filter_lte() {
    let result = ParseToken::parse("find_by_price_lte").unwrap();
    let ParseToken::FindBy(cols) = result else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::Lte]);
  }

  #[test]
  fn test_filter_between() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_date_between").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::Between]);
  }

  #[test]
  fn test_filter_not_between() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_date_not_between").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::NotBetween]);
  }

  #[test]
  fn test_filter_contains() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_description_contains").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::Contains]);
  }

  #[test]
  fn test_filter_starts_with() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_name_starts_with").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::StartsWith]);
  }

  #[test]
  fn test_filter_ends_with() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_email_ends_with").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::EndsWith]);
  }

  #[test]
  fn test_filter_like() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_pattern_like").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::Like]);
  }

  #[test]
  fn test_filter_not_like() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_pattern_not_like").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::NotLike]);
  }

  #[test]
  fn test_filter_is_null() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_deleted_at_is_null").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "deleted_at");
    assert_eq!(cols[0].filters, vec![ColFilter::IsNull]);
  }

  #[test]
  fn test_filter_is_not_null() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_email_is_not_null").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::IsNotNull]);
  }

  #[test]
  fn test_filter_is_in() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_status_is_in").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters, vec![ColFilter::IsIn]);
  }

  #[test]
  fn test_filter_is_not_in() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_status_is_not_in").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "status");
    assert_eq!(cols[0].filters, vec![ColFilter::IsNotIn]);
  }

  #[test]
  fn test_multiple_filters_on_same_column() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_age_gt_lt").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "age");
    assert_eq!(cols[0].filters, vec![ColFilter::Gt, ColFilter::Lt]);
  }

  #[test]
  fn test_multiple_columns_with_filters() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_name_like_and_age_gt").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].name, "name");
    assert_eq!(cols[0].filters, vec![ColFilter::Like]);
    assert_eq!(cols[1].name, "age");
    assert_eq!(cols[1].filters, vec![ColFilter::Gt]);
  }

  #[test]
  fn test_find_all_by() {
    let ParseToken::FindAllBy(cols) = ParseToken::parse("find_all_by_status").unwrap() else {
      panic!("expected FindAllBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "status");
  }

  #[test]
  fn test_error_missing_by() {
    let result = ParseToken::parse("find_name");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Expected 'by'"));
  }

  #[test]
  fn test_error_invalid_function_type() {
    let result = ParseToken::parse("search_by_name");
    assert!(result.is_err());
  }

  #[test]
  fn test_error_invalid_not_filter() {
    let result = ParseToken::parse("find_by_name_not_invalid");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("'not' must be followed by 'eq', 'between', or 'like'"));
  }

  #[test]
  fn test_error_incomplete_not_filter() {
    let result = ParseToken::parse("find_by_name_not");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("'not' must be followed by"));
  }

  #[test]
  fn test_error_invalid_starts_filter() {
    let result = ParseToken::parse("find_by_name_starts_at");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("'starts' must be followed by 'with'"));
  }

  #[test]
  fn test_error_incomplete_starts_filter() {
    let result = ParseToken::parse("find_by_name_starts");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("'starts' must be followed by 'with'"));
  }

  #[test]
  fn test_error_invalid_ends_filter() {
    let result = ParseToken::parse("find_by_name_ends_at");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("'ends' must be followed by 'with'"));
  }

  #[test]
  fn test_error_incomplete_ends_filter() {
    let result = ParseToken::parse("find_by_name_ends");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("'ends' must be followed by 'with'"));
  }

  #[test]
  fn test_error_invalid_is_filter() {
    let result = ParseToken::parse("find_by_status_is_invalid");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("'is' must be followed by 'null', 'not', or 'in'"));
  }

  #[test]
  fn test_error_incomplete_is_filter() {
    let result = ParseToken::parse("find_by_status_is");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("'is' must be followed by"));
  }

  #[test]
  fn test_error_invalid_is_not_filter() {
    let result = ParseToken::parse("find_by_email_is_not_empty");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("'is_not' must be followed by 'null' or 'in'"));
  }

  #[test]
  fn test_error_incomplete_is_not_filter() {
    let result = ParseToken::parse("find_by_email_is_not");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("'is_not' must be followed by 'null' or 'in'"));
  }

  #[test]
  fn test_default_filter_when_empty() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_username").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols[0].filters(), vec![ColFilter::Eq]);
  }

  #[test]
  fn test_complex_query() {
    let ParseToken::FindBy(cols) = ParseToken::parse(
      "find_by_email_starts_with_and_age_gte_and_status_is_in"
    ).unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 3);
    assert_eq!(cols[0].name, "email");
    assert_eq!(cols[0].filters, vec![ColFilter::StartsWith]);
    assert_eq!(cols[1].name, "age");
    assert_eq!(cols[1].filters, vec![ColFilter::Gte]);
    assert_eq!(cols[2].name, "status");
    assert_eq!(cols[2].filters, vec![ColFilter::IsIn]);
  }

  #[test]
  fn test_find_all_by_with_complex_filters() {
    let ParseToken::FindAllBy(cols) = ParseToken::parse(
      "find_all_by_created_at_between_and_status_not_eq"
    ).unwrap() else {
      panic!("expected FindAllBy");
    };

    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].name, "created_at");
    assert_eq!(cols[0].filters, vec![ColFilter::Between]);
    assert_eq!(cols[1].name, "status");
    assert_eq!(cols[1].filters, vec![ColFilter::NotEq]);
  }

  #[test]
  fn test_count_by_with_filters() {
    let ParseToken::CountBy(cols) = ParseToken::parse("count_by_name_like").unwrap() else {
      panic!("expected CountBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].filters, vec![ColFilter::Like]);
  }

  #[test]
  fn test_delete_by_with_filters() {
    let ParseToken::DeleteBy(cols) = ParseToken::parse("delete_by_id_is_in").unwrap() else {
      panic!("expected DeleteBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].filters, vec![ColFilter::IsIn]);
  }

  #[test]
  fn test_column_with_underscores() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_created_at").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "created_at");
    assert_eq!(cols[0].filters(), vec![ColFilter::Eq]);
  }

  #[test]
  fn test_column_with_underscores_and_filter() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_updated_at_gte").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "updated_at");
    assert_eq!(cols[0].filters, vec![ColFilter::Gte]);
  }

  #[test]
  fn test_multiple_columns_with_underscores() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_first_name_and_last_name").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].name, "first_name");
    assert_eq!(cols[1].name, "last_name");
  }

  #[test]
  fn test_column_with_underscores_multiple_filters() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_created_at_gte_lt").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "created_at");
    assert_eq!(cols[0].filters, vec![ColFilter::Gte, ColFilter::Lt]);
  }

  #[test]
  fn test_complex_with_underscores_and_filters() {
    let ParseToken::FindBy(cols) = ParseToken::parse(
      "find_by_user_email_like_and_created_at_between_and_status_code_is_not_null"
    ).unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 3);
    assert_eq!(cols[0].name, "user_email");
    assert_eq!(cols[0].filters, vec![ColFilter::Like]);
    assert_eq!(cols[1].name, "created_at");
    assert_eq!(cols[1].filters, vec![ColFilter::Between]);
    assert_eq!(cols[2].name, "status_code");
    assert_eq!(cols[2].filters, vec![ColFilter::IsNotNull]);
  }

  #[test]
  fn test_triple_underscore_column() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_some_long_column_name").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "some_long_column_name");
  }

  #[test]
  fn test_find_all_by_with_underscores() {
    let ParseToken::FindAllBy(cols) = ParseToken::parse("find_all_by_user_id").unwrap() else {
      panic!("expected FindAllBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "user_id");
  }

  #[test]
  fn test_count_by_with_underscores() {
    let ParseToken::CountBy(cols) = ParseToken::parse("count_by_account_type").unwrap() else {
      panic!("expected CountBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "account_type");
  }

  #[test]
  fn test_delete_by_with_underscores() {
    let ParseToken::DeleteBy(cols) = ParseToken::parse("delete_by_session_id").unwrap() else {
      panic!("expected DeleteBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "session_id");
  }

  #[test]
  fn test_is_not_in_with_underscores() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_user_role_is_not_in").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "user_role");
    assert_eq!(cols[0].filters, vec![ColFilter::IsNotIn]);
  }

  #[test]
  fn test_is_not_in_with_multiple_columns() {
    let ParseToken::FindBy(cols) = ParseToken::parse("find_by_status_is_not_in_and_type_eq").unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 2);
    assert_eq!(cols[0].name, "status");
    assert_eq!(cols[0].filters, vec![ColFilter::IsNotIn]);
    assert_eq!(cols[1].name, "type");
    assert_eq!(cols[1].filters, vec![ColFilter::Eq]);
  }

  #[test]
  fn test_find_all_by_is_not_in() {
    let ParseToken::FindAllBy(cols) = ParseToken::parse("find_all_by_category_is_not_in").unwrap() else {
      panic!("expected FindAllBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "category");
    assert_eq!(cols[0].filters, vec![ColFilter::IsNotIn]);
  }

  #[test]
  fn test_count_by_is_not_in() {
    let ParseToken::CountBy(cols) = ParseToken::parse("count_by_tag_is_not_in").unwrap() else {
      panic!("expected CountBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "tag");
    assert_eq!(cols[0].filters, vec![ColFilter::IsNotIn]);
  }

  #[test]
  fn test_delete_by_is_not_in() {
    let ParseToken::DeleteBy(cols) = ParseToken::parse("delete_by_status_is_not_in").unwrap() else {
      panic!("expected DeleteBy");
    };

    assert_eq!(cols.len(), 1);
    assert_eq!(cols[0].name, "status");
    assert_eq!(cols[0].filters, vec![ColFilter::IsNotIn]);
  }

  #[test]
  fn test_complex_query_with_is_not_in() {
    let ParseToken::FindBy(cols) = ParseToken::parse(
      "find_by_user_id_eq_and_status_is_not_in_and_created_at_gte"
    ).unwrap() else {
      panic!("expected FindBy");
    };

    assert_eq!(cols.len(), 3);
    assert_eq!(cols[0].name, "user_id");
    assert_eq!(cols[0].filters, vec![ColFilter::Eq]);
    assert_eq!(cols[1].name, "status");
    assert_eq!(cols[1].filters, vec![ColFilter::IsNotIn]);
    assert_eq!(cols[2].name, "created_at");
    assert_eq!(cols[2].filters, vec![ColFilter::Gte]);
  }

  #[test]
  fn test_error_double_and() {
    let result = ParseToken::parse("find_by_name_and_and_age");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Unexpected 'and'"));
  }

  #[test]
  fn test_error_starting_with_and() {
    let result = ParseToken::parse("find_by_and_name");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Unexpected 'and'"));
  }

  #[test]
  fn test_error_ending_with_and() {
    let result = ParseToken::parse("find_by_name_and");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Expected column name after 'and'"));
  }
}
