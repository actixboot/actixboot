# Repository Macro

A powerful procedural macro for generating SeaORM repository methods from trait function signatures.

## Overview

The `#[repository(entity_module)]` macro analyzes function names and generates complete SeaORM query implementations automatically. No need to write repetitive query code - just define the function signature and the macro handles the rest.

## Basic Usage

```rust
use actix_boot::repository::macros::repository;

#[repository(post)]
pub trait PostRepositoryBase {
  async fn find_by_id(&self, id: i32) -> Result<Option<post::Model>, sea_orm::DbErr>;
}
```

## Function Naming Patterns

### Query Operations

#### `find_by_*`
Returns a single record (`Option<Model>`).

```rust
async fn find_by_id(&self, id: i32) -> Result<Option<Model>, DbErr>;
async fn find_by_email(&self, email: &str) -> Result<Option<Model>, DbErr>;
async fn find_by_status_and_category(&self, status: &str, category: &str) -> Result<Option<Model>, DbErr>;
```

#### `find_all_by_*`
Returns multiple records (`Vec<Model>`).

```rust
async fn find_all_by_status(&self, status: &str) -> Result<Vec<Model>, DbErr>;
async fn find_all_by_user_id(&self, user_id: i32) -> Result<Vec<Model>, DbErr>;
```

#### `count_by_*`
Counts matching records.

```rust
async fn count_by_status(&self, status: &str) -> Result<u64, DbErr>;
async fn count_by_category(&self, category: &str) -> Result<u64, DbErr>;
```

#### `exists_by_*`
Checks if matching records exist.

```rust
async fn exists_by_email(&self, email: &str) -> Result<bool, DbErr>;
async fn exists_by_id(&self, id: i32) -> Result<bool, DbErr>;
```

#### `delete_by_*`
Deletes matching records.

```rust
async fn delete_by_id(&self, id: i32) -> Result<DeleteResult, DbErr>;
async fn delete_by_status(&self, status: &str) -> Result<DeleteResult, DbErr>;
```

### Update Operations

Pattern: `update_{column}_by_{filter}` or `update_{col1}_and_{col2}_by_{filter}`

```rust
async fn update_status_by_id(&self, id: i32, status: String) -> Result<UpdateResult, DbErr>;

async fn update_name_and_email_by_id(&self, id: i32, name: String, email: String)
  -> Result<UpdateResult, DbErr>;
```

Update parameters come first, then filter parameters.

### Aggregate Operations

#### `sum_{column}_by_*`
```rust
async fn sum_price_by_category(&self, category: &str) -> Result<Option<Decimal>, DbErr>;
```

#### `avg_{column}_by_*`
```rust
async fn avg_rating_by_product_id(&self, product_id: i32) -> Result<Option<f64>, DbErr>;
```

#### `min_{column}_by_*`
```rust
async fn min_price_by_category(&self, category: &str) -> Result<Option<Decimal>, DbErr>;
```

#### `max_{column}_by_*`
```rust
async fn max_created_at_by_user_id(&self, user_id: i32) -> Result<Option<DateTime>, DbErr>;
```

## Filters

Filters are appended to column names in function signatures.

### Equality Filters

#### `_{column}` (default eq)
```rust
async fn find_by_status(&self, status: &str) -> Result<Option<Model>, DbErr>;
```

#### `_{column}_eq`
```rust
async fn find_by_id_eq(&self, id: i32) -> Result<Option<Model>, DbErr>;
```

#### `_{column}_not_eq` or `_{column}_ne`
```rust
async fn find_all_by_status_not_eq(&self, status: &str) -> Result<Vec<Model>, DbErr>;
```

### Comparison Filters

#### `_{column}_gt` (greater than)
```rust
async fn find_all_by_price_gt(&self, price: Decimal) -> Result<Vec<Model>, DbErr>;
```

#### `_{column}_gte` (greater than or equal)
```rust
async fn find_all_by_age_gte(&self, age: i32) -> Result<Vec<Model>, DbErr>;
```

#### `_{column}_lt` (less than)
```rust
async fn find_all_by_stock_lt(&self, stock: i32) -> Result<Vec<Model>, DbErr>;
```

#### `_{column}_lte` (less than or equal)
```rust
async fn find_all_by_price_lte(&self, price: Decimal) -> Result<Vec<Model>, DbErr>;
```

### Range Filters

#### `_{column}_between`
```rust
async fn find_all_by_price_between(&self, min: Decimal, max: Decimal) -> Result<Vec<Model>, DbErr>;
```

#### `_{column}_not_between`
```rust
async fn find_all_by_age_not_between(&self, min: i32, max: i32) -> Result<Vec<Model>, DbErr>;
```

### String Filters

#### `_{column}_like`
```rust
async fn find_all_by_name_like(&self, pattern: &str) -> Result<Vec<Model>, DbErr>;
```

#### `_{column}_not_like`
```rust
async fn find_all_by_email_not_like(&self, pattern: &str) -> Result<Vec<Model>, DbErr>;
```

#### `_{column}_contains`
```rust
async fn find_all_by_description_contains(&self, text: &str) -> Result<Vec<Model>, DbErr>;
```

#### `_{column}_starts_with`
```rust
async fn find_all_by_name_starts_with(&self, prefix: &str) -> Result<Vec<Model>, DbErr>;
```

#### `_{column}_ends_with`
```rust
async fn find_all_by_email_ends_with(&self, suffix: &str) -> Result<Vec<Model>, DbErr>;
```

### Collection Filters

#### `_{column}_is_in` or `_{column}_in`
```rust
async fn find_all_by_status_is_in(&self, statuses: Vec<String>) -> Result<Vec<Model>, DbErr>;
```

#### `_{column}_is_not_in` or `_{column}_not_in`
```rust
async fn find_all_by_id_is_not_in(&self, ids: Vec<i32>) -> Result<Vec<Model>, DbErr>;
```

### Null Filters

#### `_{column}_is_null`
No parameter needed.

```rust
async fn find_all_by_deleted_at_is_null(&self) -> Result<Vec<Model>, DbErr>;
```

#### `_{column}_is_not_null`
No parameter needed.

```rust
async fn find_all_by_email_is_not_null(&self) -> Result<Vec<Model>, DbErr>;
```

## Query Modifiers

Modifiers can be chained together and appear after filters.

### Ordering

#### `_order_by_{column}_asc`
```rust
async fn find_all_by_status_order_by_created_at_asc(&self, status: &str)
  -> Result<Vec<Model>, DbErr>;
```

#### `_order_by_{column}_desc`
```rust
async fn find_all_by_category_order_by_price_desc(&self, category: &str)
  -> Result<Vec<Model>, DbErr>;
```

Multiple orderings:
```rust
async fn find_all_by_status_order_by_priority_desc_order_by_created_at_asc(&self, status: &str)
  -> Result<Vec<Model>, DbErr>;
```

### Limiting

#### `_limit`
```rust
async fn find_all_by_status_limit(&self, status: &str, limit: u64)
  -> Result<Vec<Model>, DbErr>;
```

#### `_offset`
Must be used with `_limit`.

```rust
async fn find_all_by_status_offset_limit(&self, status: &str, offset: u64, limit: u64)
  -> Result<Vec<Model>, DbErr>;
```

#### `_paginate`
```rust
async fn find_all_by_status_paginate(&self, status: &str, page: u64, per_page: u64)
  -> Result<Vec<Model>, DbErr>;
```

### Distinct

#### `_distinct`
```rust
async fn find_all_by_category_distinct(&self, category: &str)
  -> Result<Vec<Model>, DbErr>;
```

## Combining Multiple Columns

Use `_and_` to combine multiple column filters.

```rust
async fn find_by_email_and_password(&self, email: &str, password: &str)
  -> Result<Option<Model>, DbErr>;

async fn find_all_by_status_and_category(&self, status: &str, category: &str)
  -> Result<Vec<Model>, DbErr>;

async fn find_by_id_gt_and_status(&self, id: i32, status: &str)
  -> Result<Option<Model>, DbErr>;
```

## Complex Examples

### Multiple Filters + Ordering + Limit
```rust
async fn find_all_by_status_and_priority_gte_order_by_created_at_desc_limit(
  &self,
  status: &str,
  priority: i32,
  limit: u64
) -> Result<Vec<Model>, DbErr>;
```

### Range Filter + Multiple Ordering
```rust
async fn find_all_by_price_between_order_by_category_asc_order_by_price_desc(
  &self,
  min_price: Decimal,
  max_price: Decimal
) -> Result<Vec<Model>, DbErr>;
```

### String Filter + Pagination
```rust
async fn find_all_by_name_like_paginate(
  &self,
  pattern: &str,
  page: u64,
  per_page: u64
) -> Result<Vec<Model>, DbErr>;
```

### Update with Multiple Filters
```rust
async fn update_status_by_id_and_user_id(
  &self,
  id: i32,
  user_id: i32,
  status: String
) -> Result<UpdateResult, DbErr>;
```

### Aggregate with Filters
```rust
async fn sum_amount_by_user_id_and_status(
  &self,
  user_id: i32,
  status: &str
) -> Result<Option<Decimal>, DbErr>;
```

### Null Checks with Other Filters
```rust
async fn find_all_by_deleted_at_is_null_and_status(&self, status: &str)
  -> Result<Vec<Model>, DbErr>;
```

## Parameter Order

1. **Update operations**: Update values come first, then filter parameters
2. **Filter parameters**: In the order they appear in the function name
3. **Modifier parameters**: After all filter parameters (limit, offset, page, per_page)

Example:
```rust
async fn update_name_and_email_by_id_and_status_limit(
  &self,
  id: i32,        // filter 1
  status: &str,   // filter 2
  name: String,   // update 1
  email: String,  // update 2
  limit: u64      // modifier
) -> Result<UpdateResult, DbErr>;
```

## Column Name Conversion

Column names in function signatures use snake_case but are automatically converted to PascalCase for SeaORM Column enums:

- `user_id` → `UserId`
- `created_at` → `CreatedAt`
- `email_verified` → `EmailVerified`

## Supported Return Types

- `Result<Option<Model>, DbErr>` - Single record queries
- `Result<Vec<Model>, DbErr>` - Multiple record queries
- `Result<u64, DbErr>` - Count queries
- `Result<bool, DbErr>` - Exists queries
- `Result<DeleteResult, DbErr>` - Delete operations
- `Result<UpdateResult, DbErr>` - Update operations
- `Result<Option<T>, DbErr>` - Aggregate queries (T can be any numeric type)

## Notes

- The first parameter is always `&self`
- All filters except `is_null` and `is_not_null` require parameters
- Column names must match your SeaORM entity column names
- The macro generates optimized SeaORM queries with proper trait imports
