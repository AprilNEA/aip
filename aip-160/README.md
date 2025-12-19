# AIP-160 Filter Parser with SeaORM Support

A Rust implementation of [Google AIP-160](https://google.aip.dev/160) filtering standard with SeaORM integration.

## Features

- **Full AIP-160 Support** - Parse Google AIP-160 filter expressions
- **Type Safe** - Strongly typed AST and error handling
- **Zero Boilerplate** - No trait implementation required!
- **Well Tested** - Comprehensive test coverage
- **SeaORM Integration** - Convert filters directly to SeaORM conditions

## Installation

```toml
[dependencies]
# Basic parsing only
aip-160 = "0.1"

# With SeaORM v1 support (stable)
aip-160 = { version = "0.1", features = ["sea-orm"] }

# With SeaORM v2 support (RC version)
aip-160 = { version = "0.1", features = ["sea-orm-v2"] }
```

### SeaORM v2 Support

SeaORM v2 support is currently available as a release candidate (RC).

**Note**: SeaORM v2 is still in RC phase. The API may change before final release.

#### Usage for SeaORM v2

```rust
use aip_160::{parse_filter, ToSeaOrmConditionV2};
use sea_orm_v2::entity::prelude::*;

// Use ToSeaOrmConditionV2 trait for v2
let filter = parse_filter("name = \"Alice\" AND age > 18")?;
let condition = filter.to_condition::<Entity>()?;

Entity::find().filter(condition).all(db).await?;
```

#### Migration from v1 to v2

1. Update your `Cargo.toml` to use `sea-orm-v2` feature
2. Change import from `ToSeaOrmCondition` to `ToSeaOrmConditionV2`
3. Update SeaORM imports to use `sea_orm_v2`

## Quick Start

### Basic Parsing

```rust
use aip_160::parse_filter;

fn main() {
    let filter = parse_filter("name = \"John\" AND age > 18").unwrap();
    println!("{}", filter);
}
```

### SeaORM Integration with Automatic Type Conversion

```rust
use aip_160::{parse_filter, ToSeaOrmCondition};
use sea_orm::entity::prelude::*;

async fn find_users(
    db: &DatabaseConnection,
    filter_str: &str
) -> Result<Vec<Model>, DbErr> {
    // 1. Parse the filter string
    let filter = parse_filter(filter_str)?;

    // 2. Convert to SeaORM condition - pass Entity type!
    let condition = filter.to_condition::<Entity>()?;

    // 3. Use in your query
    Entity::find()
        .filter(condition)
        .all(db)
        .await
}
```

### ✨ Zero-Config Automatic Type Conversion

**The library automatically detects column types from your Entity and applies appropriate SQL casts!**

No manual configuration needed - just pass your `Entity` type to `to_condition()`:

```rust
// PostgreSQL with UUID - automatic casting!
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, column_type = "Uuid")]
    pub id: Uuid,
    pub name: String,
    #[sea_orm(column_type = "TimestampWithTimeZone")]
    pub created_at: DateTimeWithTimeZone,
}

// Just use string values - they're automatically cast to the right type!
let filter = parse_filter("id = \"550e8400-e29b-41d4-a716-446655440000\"")?;
let condition = filter.to_condition::<Entity>()?;  // Automatically casts to UUID!

Entity::find().filter(condition).all(db).await?;
```

**Supported automatic conversions:**
- `String` → `UUID` (for UUID columns)
- `String` → `TIMESTAMP` / `TIMESTAMPTZ` (for timestamp columns)
- `String` → `DATE` (for date columns)
- `String` → `TIME` (for time columns)
- `String` → `ENUM` (for PostgreSQL enum columns)
- `Number` → `INTEGER` / `FLOAT` (based on database column type)

The library reads your Entity's column definitions and generates the appropriate SQL `CAST` expressions automatically!

## Supported Filter Syntax

### Comparison Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `=` | Equal | `name = "John"` |
| `!=` | Not equal | `status != "inactive"` |
| `>` | Greater than | `age > 18` |
| `>=` | Greater than or equal | `age >= 18` |
| `<` | Less than | `age < 65` |
| `<=` | Less than or equal | `age <= 65` |
| `:` | Contains/Has | `email : "@gmail.com"` |

### Logical Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `AND` | Logical AND | `active = true AND age > 18` |
| `OR` | Logical OR | `status = "active" OR status = "pending"` |
| `NOT` | Logical NOT | `NOT deleted = true` |
| `( )` | Grouping | `(a = 1 OR a = 2) AND b = 3` |

### Value Types

| Type | Example |
|------|---------|
| String | `"value"` or `'value'` |
| Number | `42`, `3.14`, `-10`, `1e5` |
| Boolean | `true`, `false` |
| Null | `null` |

## 💡 Example Filters

```text
# Simple equality
name = "John Doe"

# Numeric comparison
age > 18 AND age < 65

# Multiple conditions
active = true AND age > 18 AND verified = true

# OR conditions
status = "active" OR status = "pending"

# Grouped expressions
(status = "active" OR status = "pending") AND age > 18

# Contains/substring matching
email : "@example.com"

# NOT operator
NOT deleted = true

# Complex query
(active = true OR status = "trial") AND age >= 18 AND email : "@company.com"
```

## How It Works

The library automatically converts snake_case field names to PascalCase Column names:

```text
"created_at" → Column::CreatedAt
"user_id" → Column::UserId
"is_active" → Column::IsActive
```

## Architecture

```
┌─────────────┐
│ Filter Text │  "age > 18 AND active = true"
└──────┬──────┘
       │ parse_filter()
       ▼
┌─────────────┐
│    AST      │  Filter { expression: And(...) }
└──────┬──────┘
       │ to_condition::<Column>()
       ▼
┌─────────────┐
│  Condition  │  SeaORM Condition
└─────────────┘
```


## Google AIP-160 Compliance

- ✅ Comparison operators (=, !=, <, <=, >, >=)
- ✅ Logical operators (AND, OR, NOT)
- ✅ Grouping with parentheses
- ✅ String, number, boolean, and null values
- ✅ Has/contains operator (:)
- ⚠️ Partial: Function calls (not yet implemented)

## License

MIT or Apache-2.0

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## References

- [Google AIP-160: Filtering](https://google.aip.dev/160)
- [SeaORM Documentation](https://www.sea-ql.org/SeaORM/)
- [Pest Parser](https://pest.rs/)
