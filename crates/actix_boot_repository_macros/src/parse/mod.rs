use std::iter::Peekable;
use std::slice::Iter;
use quote::quote;

#[derive(Debug)]
pub enum ParseToken {
  FindBy(QuerySpec),
  FindAllBy(QuerySpec),
  CountBy(QuerySpec),
  DeleteBy(QuerySpec),
  ExistsBy(QuerySpec),
  UpdateBy(UpdateSpec),
  SumBy(AggregateSpec),
  AvgBy(AggregateSpec),
  MinBy(AggregateSpec),
  MaxBy(AggregateSpec),
}

#[derive(Debug, Clone)]
pub struct QuerySpec {
  pub filters: Vec<ParseTokenCol>,
  pub modifiers: Vec<QueryModifier>,
}

#[derive(Debug, Clone)]
pub struct UpdateSpec {
  pub updates: Vec<String>,
  pub filters: Vec<ParseTokenCol>,
  pub modifiers: Vec<QueryModifier>,
}

#[derive(Debug, Clone)]
pub struct AggregateSpec {
  pub column: String,
  pub filters: Vec<ParseTokenCol>,
}

#[derive(Debug, Clone)]
pub enum QueryModifier {
  OrderBy(String, OrderDirection),
  Limit,
  Offset,
  Paginate,
  Distinct,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderDirection {
  Asc,
  Desc,
}

#[derive(Debug, Clone)]
pub struct ParseTokenCol {
  pub name: String,
  pub filters: Vec<ColFilter>
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
  Delete,
  Exists,
  Update,
  Sum,
  Avg,
  Min,
  Max,
}

impl FnType {
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
      "exists" => Ok(Self::Exists),
      "update" => Ok(Self::Update),
      "sum" => Ok(Self::Sum),
      "avg" => Ok(Self::Avg),
      "min" => Ok(Self::Min),
      "max" => Ok(Self::Max),
      _ => Err(Self::err(token)),
    }
  }

  fn err(token: &str) -> syn::Error {
    syn::Error::new(
      proc_macro2::Span::call_site(),
      format!("Unknown function type '{}'. Expected: find, find_all, count, delete, exists, update, sum, avg, min, max", token)
    )
  }
}

impl ParseToken {
  pub fn parse(function_name: &str) -> syn::Result<Self> {
    let split = function_name.split("_").collect::<Vec<_>>();
    let mut iter = split.iter().peekable();
    let fn_type = FnType::parse(&mut iter, function_name)?;

    match fn_type {
      FnType::Update => Self::parse_update(&mut iter),
      FnType::Sum | FnType::Avg | FnType::Min | FnType::Max => {
        Self::parse_aggregate(&mut iter, fn_type)
      },
      _ => Self::parse_query(&mut iter, fn_type),
    }
  }

  fn parse_update(iter: &mut Peekable<Iter<&str>>) -> syn::Result<Self> {
    let updates = Self::parse_column_list(iter)?;

    if updates.is_empty() {
      return Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "Update operations require at least one column to update (e.g., update_status_by_id)"
      ));
    }

    if iter.peek() != Some(&&"by") {
      return Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "Expected 'by' after update columns"
      ));
    }
    iter.next();

    let filters = Self::parse_filter_columns(iter)?;
    let modifiers = Self::parse_modifiers(iter)?;

    Ok(ParseToken::UpdateBy(UpdateSpec {
      updates,
      filters,
      modifiers,
    }))
  }

  fn parse_aggregate(iter: &mut Peekable<Iter<&str>>, fn_type: FnType) -> syn::Result<Self> {
    let columns = Self::parse_column_list(iter)?;

    if columns.len() != 1 {
      return Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        format!("Aggregate functions require exactly one column, found {}", columns.len())
      ));
    }

    let column = columns[0].clone();

    if iter.peek() != Some(&&"by") {
      return Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "Expected 'by' after aggregate column"
      ));
    }
    iter.next();

    let filters = Self::parse_filter_columns(iter)?;

    let spec = AggregateSpec { column, filters };

    Ok(match fn_type {
      FnType::Sum => ParseToken::SumBy(spec),
      FnType::Avg => ParseToken::AvgBy(spec),
      FnType::Min => ParseToken::MinBy(spec),
      FnType::Max => ParseToken::MaxBy(spec),
      _ => unreachable!(),
    })
  }

  fn parse_query(iter: &mut Peekable<Iter<&str>>, fn_type: FnType) -> syn::Result<Self> {
    if iter.peek() != Some(&&"by") {
      return Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "Expected 'by' after function type"
      ));
    }
    iter.next();

    let filters = Self::parse_filter_columns(iter)?;
    let modifiers = Self::parse_modifiers(iter)?;

    let spec = QuerySpec { filters, modifiers };

    Ok(match fn_type {
      FnType::Find => ParseToken::FindBy(spec),
      FnType::FindAll => ParseToken::FindAllBy(spec),
      FnType::Count => ParseToken::CountBy(spec),
      FnType::Delete => ParseToken::DeleteBy(spec),
      FnType::Exists => ParseToken::ExistsBy(spec),
      _ => unreachable!(),
    })
  }

  fn parse_column_list(iter: &mut Peekable<Iter<&str>>) -> syn::Result<Vec<String>> {
    let mut columns = Vec::new();

    loop {
      let part = match iter.peek() {
        Some(&&"by") | Some(&&"and") | None => break,
        Some(p) => *p,
      };

      if Self::is_modifier_keyword(part) {
        break;
      }

      let mut col_parts: Vec<&str> = vec![part];
      iter.next();

      while let Some(&&next_part) = iter.peek() {
        if next_part == "and" || next_part == "by" || Self::is_modifier_keyword(next_part) {
          break;
        }
        col_parts.push(next_part);
        iter.next();
      }

      columns.push(col_parts.join("_"));

      if iter.peek() == Some(&&"and") {
        iter.next();
      } else {
        break;
      }
    }

    Ok(columns)
  }

  fn parse_filter_columns(iter: &mut Peekable<Iter<&str>>) -> syn::Result<Vec<ParseTokenCol>> {
    let mut columns = vec![];

    loop {
      let part = match iter.peek() {
        Some(p) if Self::is_modifier_keyword(p) => break,
        Some(&&"and") => {
          return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "Unexpected 'and' - expected column name"
          ));
        },
        Some(p) => p,
        None => break,
      };

      let mut col_name_parts: Vec<&str> = vec![*part];
      iter.next();
      let mut filters = vec![];
      let mut saw_and = false;

      loop {
        if let Some(filter) = ColFilter::parse(iter)? {
          filters.push(filter);
          continue;
        }

        match iter.peek() {
          Some(p) if Self::is_modifier_keyword(p) => break,
          Some(&&"and") => {
            iter.next();
            saw_and = true;
            break;
          }
          Some(&next_part) => {
            col_name_parts.push(next_part);
            iter.next();
          }
          None => break,
        }
      }

      columns.push(ParseTokenCol {
        name: col_name_parts.join("_"),
        filters,
      });

      if saw_and && matches!(iter.peek(), None | Some(&&"order") | Some(&&"limit") | Some(&&"offset") | Some(&&"paginate") | Some(&&"distinct")) {
        return Err(syn::Error::new(
          proc_macro2::Span::call_site(),
          "Expected column name after 'and'"
        ));
      }

      if !saw_and {
        break;
      }
    }

    Ok(columns)
  }

  fn parse_modifiers(iter: &mut Peekable<Iter<&str>>) -> syn::Result<Vec<QueryModifier>> {
    let mut modifiers = Vec::new();

    while let Some(&&keyword) = iter.peek() {
      match keyword {
        "order" => {
          iter.next();
          if iter.peek() != Some(&&"by") {
            return Err(syn::Error::new(
              proc_macro2::Span::call_site(),
              "Expected 'by' after 'order'"
            ));
          }
          iter.next();

          loop {
            let mut col_parts = Vec::new();

            while let Some(&&part) = iter.peek() {
              if part == "asc" || part == "desc" || part == "and" || Self::is_modifier_keyword(part) {
                break;
              }
              col_parts.push(part);
              iter.next();
            }

            if col_parts.is_empty() {
              return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "Expected column name after 'order_by'"
              ));
            }

            let col_name = col_parts.join("_");

            let direction = match iter.peek() {
              Some(&&"asc") => {
                iter.next();
                OrderDirection::Asc
              },
              Some(&&"desc") => {
                iter.next();
                OrderDirection::Desc
              },
              _ => {
                return Err(syn::Error::new(
                  proc_macro2::Span::call_site(),
                  format!("Expected 'asc' or 'desc' after column '{}' in order_by", col_name)
                ));
              }
            };

            modifiers.push(QueryModifier::OrderBy(col_name, direction));

            if iter.peek() == Some(&&"and") {
              iter.next();
            } else {
              break;
            }
          }
        },
        "limit" => {
          iter.next();
          modifiers.push(QueryModifier::Limit);
        },
        "offset" => {
          iter.next();
          modifiers.push(QueryModifier::Offset);
        },
        "paginate" => {
          iter.next();
          modifiers.push(QueryModifier::Paginate);
        },
        "distinct" => {
          iter.next();
          modifiers.push(QueryModifier::Distinct);
        },
        _ => break,
      }
    }

    Ok(modifiers)
  }

  fn is_modifier_keyword(word: &str) -> bool {
    matches!(word, "order" | "limit" | "offset" | "paginate" | "distinct")
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

  pub fn needs_param(&self) -> bool {
    !matches!(self, ColFilter::IsNull | ColFilter::IsNotNull)
  }

  fn parse(iter: &mut Peekable<Iter<&str>>) -> syn::Result<Option<ColFilter>> {
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
