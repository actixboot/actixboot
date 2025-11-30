# Repository Macro Implementation Plan

## 📋 Current Status

### ✅ Completed Features

#### Core Query Types
- [x] `find_by_*` - Find single record with filters
- [x] `find_all_by_*` - Find all records matching filters
- [x] `count_by_*` - Count records matching filters
- [x] `delete_by_*` - Delete records matching filters

#### Filter Support
All SeaORM comparison operators:
- [x] `eq` - Equals
- [x] `not_eq` - Not equals
- [x] `gt` - Greater than
- [x] `gte` - Greater than or equal
- [x] `lt` - Less than
- [x] `lte` - Less than or equal
- [x] `between` - Between two values
- [x] `not_between` - Not between two values

String operations:
- [x] `contains` - String contains
- [x] `starts_with` - String starts with
- [x] `ends_with` - String ends with
- [x] `like` - SQL LIKE pattern
- [x] `not_like` - SQL NOT LIKE pattern

Collection operations:
- [x] `is_in` - Value in collection
- [x] `is_not_in` - Value not in collection

Null checks:
- [x] `is_null` - Column is NULL
- [x] `is_not_null` - Column is NOT NULL

#### Additional Features
- [x] Multiple filters per column (e.g., `find_by_id_gte_lte`)
- [x] Multiple columns with filters (e.g., `find_by_name_and_age_gt`)
- [x] Snake_case to PascalCase conversion for SeaORM Column enums
- [x] Async/await support
- [x] Proper parameter extraction and validation
- [x] Modular architecture (filter, query, params modules)

---

## 🎯 Phase 1: Essential Features (High Priority)

### 1.1 Ordering/Sorting ⭐⭐⭐⭐⭐
**Priority:** CRITICAL
**Complexity:** Low
**Impact:** High

#### Syntax Design
```rust
// Single column ordering
async fn find_all_by_status_order_by_created_at_desc(&self, status: &str)
  -> Result<Vec<Model>, DbErr>;

// Multiple column ordering
async fn find_all_order_by_created_at_desc_id_asc(&self)
  -> Result<Vec<Model>, DbErr>;

// With filters
async fn find_all_by_category_and_active_order_by_price_asc(&self, category: &str, active: bool)
  -> Result<Vec<Model>, DbErr>;
```

#### Implementation Tasks
- [ ] Parse `order_by` keyword in function name
- [ ] Support `asc` and `desc` directions
- [ ] Handle multiple `order_by` clauses
- [ ] Generate `.order_by_asc()` / `.order_by_desc()` calls
- [ ] Add integration tests

#### Generated Code Example
```rust
post::Entity::find()
    .filter(post::Column::Status.eq(status))
    .order_by_desc(post::Column::CreatedAt)
    .all(&self.db)
    .await
```

---

### 1.2 Pagination ⭐⭐⭐⭐⭐
**Priority:** CRITICAL
**Complexity:** Low
**Impact:** High

#### Syntax Design
```rust
// Limit only
async fn find_all_by_status_limit(&self, status: &str, limit: u64)
  -> Result<Vec<Model>, DbErr>;

// Offset + Limit
async fn find_all_by_status_offset_limit(&self, status: &str, offset: u64, limit: u64)
  -> Result<Vec<Model>, DbErr>;

// Pagination helper (page is 0-based or 1-based - decide!)
async fn find_all_by_status_paginate(&self, status: &str, page: u64, per_page: u64)
  -> Result<Vec<Model>, DbErr>;
```

#### Implementation Tasks
- [ ] Parse `limit`, `offset`, `paginate` keywords
- [ ] Add limit/offset as last parameters
- [ ] Generate `.limit()` and `.offset()` calls
- [ ] Decide on 0-based vs 1-based pagination
- [ ] Add validation (page > 0, per_page > 0)
- [ ] Add integration tests

#### Generated Code Example
```rust
// limit
post::Entity::find()
    .filter(post::Column::Status.eq(status))
    .limit(limit)
    .all(&self.db)
    .await

// paginate (assuming 1-based)
post::Entity::find()
    .filter(post::Column::Status.eq(status))
    .offset((page - 1) * per_page)
    .limit(per_page)
    .all(&self.db)
    .await
```

---

### 1.3 Update Operations ⭐⭐⭐⭐⭐
**Priority:** CRITICAL
**Complexity:** Medium
**Impact:** High

#### Syntax Design
```rust
// Update single column
async fn update_status_by_id(&self, id: i32, status: String)
  -> Result<UpdateResult, DbErr>;

// Update multiple columns
async fn update_title_and_text_by_id(&self, id: i32, title: String, text: String)
  -> Result<UpdateResult, DbErr>;

// Update with complex filters
async fn update_status_by_category_and_active(&self,
    category: &str,
    active: bool,
    new_status: String)
  -> Result<UpdateResult, DbErr>;
```

#### Implementation Tasks
- [ ] Add `update_*_by_*` pattern parsing
- [ ] Parse columns to update (between `update_` and `_by_`)
- [ ] Parse filter columns (after `_by_`)
- [ ] Generate `Entity::update_many()` queries
- [ ] Use `.col_expr()` for setting values
- [ ] Handle parameter ordering (updates first, then filters)
- [ ] Add integration tests

#### Generated Code Example
```rust
use sea_orm::sea_query::Expr;

post::Entity::update_many()
    .col_expr(post::Column::Status, Expr::value(status))
    .filter(post::Column::Id.eq(id))
    .exec(&self.db)
    .await
```

---

### 1.4 Exists Checks ⭐⭐⭐⭐
**Priority:** HIGH
**Complexity:** Low
**Impact:** Medium

#### Syntax Design
```rust
async fn exists_by_email(&self, email: &str) -> Result<bool, DbErr>;
async fn exists_by_username_and_deleted_at_is_null(&self, username: &str) -> Result<bool, DbErr>;
```

#### Implementation Tasks
- [ ] Add `exists_by_*` pattern parsing
- [ ] Generate count query and convert to boolean
- [ ] Add integration tests

#### Generated Code Example
```rust
post::Entity::find()
    .filter(post::Column::Email.eq(email))
    .count(&self.db)
    .await
    .map(|count| count > 0)
```

---

## 🚀 Phase 2: Quality of Life Features (Medium Priority)

### 2.1 Distinct Queries ⭐⭐⭐
**Priority:** MEDIUM
**Complexity:** Low
**Impact:** Medium

#### Syntax Design
```rust
async fn find_all_by_category_distinct(&self, category: &str)
  -> Result<Vec<Model>, DbErr>;
```

#### Implementation Tasks
- [ ] Parse `distinct` keyword
- [ ] Generate `.distinct()` call
- [ ] Works with order_by and pagination
- [ ] Add integration tests

---

### 2.2 Column Selection (Partial Loading) ⭐⭐⭐
**Priority:** MEDIUM
**Complexity:** Medium
**Impact:** Medium

#### Syntax Design
```rust
// Select specific columns
async fn find_all_by_status_select_id_title(&self, status: &str)
  -> Result<Vec<(i32, String)>, DbErr>;

// Single column
async fn find_all_by_category_select_price(&self, category: &str)
  -> Result<Vec<Decimal>, DbErr>;
```

#### Implementation Tasks
- [ ] Parse `select_*` suffix
- [ ] Determine return type based on selected columns
- [ ] Generate `.select_only()` with `.column()` calls
- [ ] Use `.into_tuple()` or `.into_values()` appropriately
- [ ] Handle type inference for return types
- [ ] Add integration tests

---

### 2.3 Aggregations ⭐⭐⭐
**Priority:** MEDIUM
**Complexity:** Medium
**Impact:** Medium

#### Syntax Design
```rust
async fn sum_price_by_category(&self, category: &str)
  -> Result<Option<Decimal>, DbErr>;

async fn avg_rating_by_product(&self, product_id: i32)
  -> Result<Option<f64>, DbErr>;

async fn min_price_by_category(&self, category: &str)
  -> Result<Option<Decimal>, DbErr>;

async fn max_created_at_by_user(&self, user_id: i32)
  -> Result<Option<DateTime>, DbErr>;

// Count is already supported via count_by_*
```

#### Implementation Tasks
- [ ] Parse `sum_*_by_*`, `avg_*_by_*`, `min_*_by_*`, `max_*_by_*` patterns
- [ ] Extract aggregation column
- [ ] Generate appropriate aggregation query
- [ ] Return `Option<T>` for nullable results
- [ ] Add integration tests

#### Generated Code Example
```rust
use sea_orm::sea_query::Expr;

post::Entity::find()
    .filter(post::Column::Category.eq(category))
    .select_only()
    .column_as(post::Column::Price.sum(), "sum")
    .into_tuple::<Option<Decimal>>()
    .one(&self.db)
    .await
```

---

## 🔮 Phase 3: Advanced Features (Lower Priority)

### 3.1 OR Conditions ⭐⭐
**Priority:** LOW
**Complexity:** High
**Impact:** Medium

#### Syntax Design
```rust
// Simple OR
async fn find_by_email_or_username(&self, email: &str, username: &str)
  -> Result<Option<Model>, DbErr>;

// Complex: (A AND B) OR (C AND D)
// This becomes very complex - might need different approach
```

#### Implementation Tasks
- [ ] Parse `_or_` keyword between conditions
- [ ] Use `Condition::any()` for OR
- [ ] Handle precedence with AND conditions
- [ ] Might need parentheses in function names? `find_by_(a_and_b)_or_(c_and_d)`
- [ ] Add integration tests

#### Challenges
- Complex precedence handling
- Function names can get very long
- May need to limit complexity

---

### 3.2 Locking ⭐
**Priority:** LOW
**Complexity:** Low
**Impact:** Low

#### Syntax Design
```rust
async fn find_by_id_for_update(&self, id: i32)
  -> Result<Option<Model>, DbErr>;

async fn find_by_id_for_share(&self, id: i32)
  -> Result<Option<Model>, DbErr>;
```

#### Implementation Tasks
- [ ] Parse `for_update` and `for_share` suffixes
- [ ] Generate `.lock(LockType::Update)` or `.lock_shared()`
- [ ] Add integration tests

---

### 3.3 Batch Operations ⭐
**Priority:** LOW
**Complexity:** Medium
**Impact:** Low

#### Syntax Design
```rust
async fn insert_many(&self, models: Vec<ActiveModel>)
  -> Result<InsertResult<ActiveModel>, DbErr>;

async fn upsert_many(&self, models: Vec<ActiveModel>)
  -> Result<InsertResult<ActiveModel>, DbErr>;
```

#### Implementation Tasks
- [ ] Not pattern-based - might be better as trait methods
- [ ] Consider adding to base Repository trait instead

---

### 3.4 Custom SQL Functions ⭐
**Priority:** LOW
**Complexity:** High
**Impact:** Low

#### Syntax Design
```rust
// String functions
async fn find_all_by_email_lower_eq(&self, email: &str)
  -> Result<Vec<Model>, DbErr>;

async fn find_all_by_name_upper_starts_with(&self, prefix: &str)
  -> Result<Vec<Model>, DbErr>;

// Date functions
async fn find_all_by_created_at_date_eq(&self, date: NaiveDate)
  -> Result<Vec<Model>, DbErr>;
```

#### Implementation Tasks
- [ ] Parse function names (lower, upper, date, year, month, etc.)
- [ ] Generate SQL function calls
- [ ] Very database-specific - needs careful handling

---

## 📝 Implementation Strategy

### Order of Implementation

**Sprint 1 (Week 1):**
1. Ordering support (`order_by_*_asc/desc`)
2. Basic pagination (`limit`, `offset`)

**Sprint 2 (Week 2):**
3. Update operations (`update_*_by_*`)
4. Exists checks (`exists_by_*`)

**Sprint 3 (Week 3):**
5. Distinct queries
6. Advanced pagination (`paginate`)

**Sprint 4 (Week 4):**
7. Column selection (`select_*`)
8. Basic aggregations (`sum`, `avg`, `min`, `max`)

**Future Consideration:**
9. OR conditions (if there's demand)
10. Locking (specialized use cases)
11. Custom SQL functions (advanced users)

---

## 🧪 Testing Strategy

For each feature:
- [ ] Unit tests for parsing logic
- [ ] Integration tests with actual database
- [ ] Test with multiple column types
- [ ] Test error cases (invalid syntax, type mismatches)
- [ ] Test combination with existing features

Example test coverage:
```rust
#[test]
fn test_parse_order_by_single_column_asc() { }

#[test]
fn test_parse_order_by_multiple_columns() { }

#[test]
fn test_order_by_with_filters() { }

#[test]
fn test_limit_with_order_by() { }

#[test]
fn test_paginate_with_filters_and_ordering() { }
```

---

## 🎨 Code Quality Goals

- **Modularity:** Each feature in its own module/function
- **Testability:** Every feature has comprehensive tests
- **Documentation:** Examples for every pattern
- **Error Messages:** Clear, helpful error messages
- **Performance:** No runtime overhead, all compile-time

---

## 📚 Documentation Plan

### README.md Updates
- [ ] Add examples for each feature
- [ ] Create comparison table (before/after)
- [ ] Add cookbook section
- [ ] Document limitations

### Examples
- [ ] Create comprehensive example app
- [ ] Show common patterns
- [ ] Demonstrate complex queries

### API Documentation
- [ ] Document all supported patterns
- [ ] Provide migration guide
- [ ] Add troubleshooting section

---

## 🚧 Known Limitations & Future Ideas

### Current Limitations
- No join support (would need relation definitions)
- No grouping (GROUP BY)
- No HAVING clauses
- No subqueries
- No raw SQL mixing

### Future Exploration
- **Compile-time SQL validation** - Verify columns exist
- **Type-safe select** - Return custom structs for partial loads
- **Query composition** - Build queries from parts
- **Relation loading** - `find_with_comments`, `find_with_author`
- **Custom derive for Column enums** - Auto-generate from database schema

---

## 🤝 Contributing

When implementing new features:
1. Update this plan with implementation details
2. Mark tasks as completed with `[x]`
3. Add any discovered issues or improvements
4. Update documentation
5. Add comprehensive tests

---

**Last Updated:** 2025-11-30
**Current Phase:** Phase 1 - Essential Features
**Next Milestone:** Ordering + Pagination support
