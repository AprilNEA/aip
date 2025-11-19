/*!
# Google AIP-160 Filter Parser with SeaORM Support

This crate provides a parser for [Google AIP-160](https://google.aip.dev/160) filtering expressions
and converts them to SeaORM conditions for database queries.

## Features

- Parse Google AIP-160 filter expressions
- Convert filters to SeaORM conditions
- Support for logical operators (AND, OR, NOT)
- Support for comparison operators (=, !=, <, <=, >, >=, :)
- Support for strings, numbers, booleans, and null values

## Usage

### Parsing a Filter

```rust
use aip_160::parse_filter;

let filter = parse_filter("name = \"John\" AND age > 18").unwrap();
println!("{}", filter);
```

### Converting to SeaORM Condition

```rust,ignore
use sea_orm::entity::prelude::*;
use aip_160::{parse_filter, sea_orm::{FieldMapper, ToSeaOrmCondition}};

// Define your entity
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
    pub active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

// Implement the FieldMapper trait
impl FieldMapper for Entity {
    fn map_field(&self, field: &str) -> aip_160::error::Result<sea_orm::sea_query::SimpleExpr> {
        match field {
            "id" => Ok(Column::Id.into_simple_expr()),
            "name" => Ok(Column::Name.into_simple_expr()),
            "email" => Ok(Column::Email.into_simple_expr()),
            "age" => Ok(Column::Age.into_simple_expr()),
            "active" => Ok(Column::Active.into_simple_expr()),
            _ => Err(aip_160::error::FilterError::InvalidField(field.to_string())),
        }
    }
}

// Use in your query
async fn find_users(db: &DatabaseConnection, filter_str: &str) -> Result<Vec<Model>, DbErr> {
    let filter = parse_filter(filter_str).unwrap();
    let condition = filter.to_condition(&Entity).unwrap();

    Entity::find()
        .filter(condition)
        .all(db)
        .await
}
```

## Supported Filter Syntax

### Basic Comparisons
- `name = "John"` - Equality
- `age > 18` - Greater than
- `age >= 18` - Greater than or equal
- `age < 65` - Less than
- `age <= 65` - Less than or equal
- `status != "inactive"` - Not equal
- `tags : "important"` - Contains/has (for substring matching)

### Logical Operators
- `name = "John" AND age > 18` - AND
- `status = "active" OR status = "pending"` - OR
- `NOT deleted = true` - NOT
- `(status = "active" OR status = "pending") AND age > 18` - Grouped expressions

### Value Types
- Strings: `"value"` or `'value'`
- Numbers: `42`, `3.14`, `-10`, `1e5`
- Booleans: `true`, `false` (case insensitive)
- Null: `null` (case insensitive)

## Example Filters

```text
name = "John Doe"
age > 18 AND active = true
(status = "active" OR status = "pending") AND created > "2024-01-01"
NOT deleted = true
tags : "important"
price >= 100 AND price <= 1000
email : "@example.com" AND verified = true
```
*/

pub mod ast;
pub mod error;
pub mod parser;

#[cfg(feature = "sea-orm")]
pub mod sea_orm;

// Re-export commonly used items
pub use ast::{Comparator, Expression, Filter, Restriction, Value};
pub use error::{FilterError, Result};
pub use parser::parse_filter;

#[cfg(feature = "sea-orm")]
pub use sea_orm::{column_from_str, ToSeaOrmCondition};
