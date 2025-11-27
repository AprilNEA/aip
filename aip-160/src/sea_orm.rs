/// SeaORM integration
#[cfg(feature = "sea-orm")]
use sea_orm::{
    sea_query::{SimpleExpr, ExprTrait, Expr, Alias, ColumnType as SeaColumnType},
    Condition, EntityTrait, ColumnTrait, IntoSimpleExpr,
};

use crate::ast::{Comparator, Expression, Filter, Restriction, Value};
use crate::error::{FilterError, Result};

/// Helper function to convert a field name to a column
///
/// Supports both exact matches and snake_case to PascalCase conversion.
pub fn column_from_str<C>(field: &str) -> Result<SimpleExpr>
where
    C: std::str::FromStr + sea_orm::IntoSimpleExpr,
    <C as std::str::FromStr>::Err: std::fmt::Display,
{
    // Try direct conversion first
    if let Ok(column) = field.parse::<C>() {
        return Ok(column.into_simple_expr());
    }

    // Try PascalCase conversion: "user_name" -> "UserName"
    let pascal_case = field
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<String>();

    pascal_case
        .parse::<C>()
        .map(|c| c.into_simple_expr())
        .map_err(|e| FilterError::InvalidField(format!("{}: {}", field, e)))
}

/// Convert a filter to a SeaORM Condition
///
/// # Example
///
/// ```ignore
/// use aip_160::{parse_filter, ToSeaOrmCondition};
/// use sea_orm::entity::prelude::*;
///
/// let filter = parse_filter("name = \"Alice\" AND age > 18")?;
/// let condition = filter.to_condition::<Entity>()?;
///
/// Entity::find().filter(condition).all(db).await?;
/// ```
pub trait ToSeaOrmCondition {
    /// Convert filter to condition with automatic type inference from Entity
    ///
    /// # Example
    ///
    /// ```ignore
    /// let filter = parse_filter("id = \"550e8400-e29b-41d4-a716-446655440000\"")?;
    /// let condition = filter.to_condition::<Entity>()?;
    /// Entity::find().filter(condition).all(db).await?;
    /// ```
    fn to_condition<E>(&self) -> Result<Condition>
    where
        E: EntityTrait,
        <E as EntityTrait>::Column: std::str::FromStr + ColumnTrait,
        <<E as EntityTrait>::Column as std::str::FromStr>::Err: std::fmt::Display;
}

impl ToSeaOrmCondition for Filter {
    fn to_condition<E>(&self) -> Result<Condition>
    where
        E: EntityTrait,
        <E as EntityTrait>::Column: std::str::FromStr + ColumnTrait,
        <<E as EntityTrait>::Column as std::str::FromStr>::Err: std::fmt::Display,
    {
        expression_to_condition::<E>(&self.expression)
    }
}

fn expression_to_condition<E>(expr: &Expression) -> Result<Condition>
where
    E: EntityTrait,
    <E as EntityTrait>::Column: std::str::FromStr + ColumnTrait,
    <<E as EntityTrait>::Column as std::str::FromStr>::Err: std::fmt::Display,
{
    match expr {
        Expression::And(left, right) => {
            let left_cond = expression_to_condition::<E>(left)?;
            let right_cond = expression_to_condition::<E>(right)?;
            Ok(Condition::all().add(left_cond).add(right_cond))
        }
        Expression::Or(left, right) => {
            let left_cond = expression_to_condition::<E>(left)?;
            let right_cond = expression_to_condition::<E>(right)?;
            Ok(Condition::any().add(left_cond).add(right_cond))
        }
        Expression::Not(inner) => {
            let inner_cond = expression_to_condition::<E>(inner)?;
            Ok(inner_cond.not())
        }
        Expression::Restriction(restriction) => {
            restriction_to_condition::<E>(restriction)
        }
        Expression::Sequence(_) => {
            Err(FilterError::UnsupportedOperation(
                "Sequences are not yet supported in SeaORM conversion".to_string(),
            ))
        }
    }
}

/// Helper to get the Column instance from field name
fn get_column<C>(field: &str) -> Result<C>
where
    C: std::str::FromStr,
    <C as std::str::FromStr>::Err: std::fmt::Display,
{
    // Try direct conversion first
    if let Ok(column) = field.parse::<C>() {
        return Ok(column);
    }

    // Try PascalCase conversion
    let pascal_case = field
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<String>();

    pascal_case
        .parse::<C>()
        .map_err(|e| FilterError::InvalidField(format!("{}: {}", field, e)))
}

fn restriction_to_condition<E>(restriction: &Restriction) -> Result<Condition>
where
    E: EntityTrait,
    <E as EntityTrait>::Column: std::str::FromStr + ColumnTrait,
    <<E as EntityTrait>::Column as std::str::FromStr>::Err: std::fmt::Display,
{
    let column_obj = get_column::<<E as EntityTrait>::Column>(&restriction.field)?;
    let column = column_obj.into_simple_expr();

    // Get column type from the Entity Column definition for automatic casting
    let col_def = column_obj.def();
    let col_type = col_def.get_column_type();

    // Apply appropriate casting based on column type
    let value_expr: SimpleExpr = match (&restriction.value, col_type) {
        // UUID columns - cast string to uuid
        (Value::String(s), SeaColumnType::Uuid) => {
            Expr::val(s.as_str()).cast_as(Alias::new("uuid")).into()
        }
        // Timestamp columns - cast string to timestamp
        (Value::String(s), SeaColumnType::TimestampWithTimeZone) => {
            Expr::val(s.as_str()).cast_as(Alias::new("timestamptz")).into()
        }
        (Value::String(s), SeaColumnType::Timestamp) => {
            Expr::val(s.as_str()).cast_as(Alias::new("timestamp")).into()
        }
        // Date columns
        (Value::String(s), SeaColumnType::Date) => {
            Expr::val(s.as_str()).cast_as(Alias::new("date")).into()
        }
        // Time columns
        (Value::String(s), SeaColumnType::Time) => {
            Expr::val(s.as_str()).cast_as(Alias::new("time")).into()
        }
        // Enum columns - cast string to the enum type
        (Value::String(s), SeaColumnType::Enum { name, .. }) => {
            Expr::val(s.as_str()).cast_as(Alias::new(name.to_string())).into()
        }
        // Default: no explicit cast needed
        (Value::String(s), _) => Expr::val(s.as_str()).into(),
        (Value::Number(n), _) => {
            if n.fract() == 0.0 {
                Expr::val(n.trunc() as i64).into()
            } else {
                Expr::val(*n).into()
            }
        }
        (Value::Boolean(b), _) => Expr::val(*b).into(),
        (Value::Null, _) => Expr::value(sea_orm::sea_query::Value::String(None)).into(),
    };

    let condition = match (&restriction.comparator, &restriction.value) {
        (Comparator::Equal, Value::Null) => Condition::all().add(column.is_null()),
        (Comparator::NotEqual, Value::Null) => Condition::all().add(column.is_not_null()),
        (Comparator::Has, Value::String(s)) => Condition::all().add(column.like(format!("%{}%", s))),
        (Comparator::Has, _) => {
            return Err(FilterError::UnsupportedOperation(
                "Has operator (:) requires a string value".to_string()
            ))
        }
        (Comparator::Equal, _) => Condition::all().add(Expr::expr(column).eq(value_expr)),
        (Comparator::NotEqual, _) => Condition::all().add(Expr::expr(column).ne(value_expr)),
        (Comparator::GreaterThan, _) => Condition::all().add(Expr::expr(column).gt(value_expr)),
        (Comparator::GreaterThanOrEqual, _) => Condition::all().add(Expr::expr(column).gte(value_expr)),
        (Comparator::LessThan, _) => Condition::all().add(Expr::expr(column).lt(value_expr)),
        (Comparator::LessThanOrEqual, _) => Condition::all().add(Expr::expr(column).lte(value_expr)),
    };

    Ok(condition)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Unit tests here are minimal because full Entity integration
    // requires complex trait implementations. See tests/sea_orm_integration.rs
    // for comprehensive integration tests with a real SeaORM Entity.

    #[test]
    fn test_column_name_conversion() {
        // Test snake_case to PascalCase conversion helper
        let result = get_column::<MockColumn>("user_name");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MockColumn::UserName);

        let result = get_column::<MockColumn>("created_at");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MockColumn::CreatedAt);
    }

    #[test]
    fn test_column_name_direct_match() {
        // Test direct PascalCase match
        let result = get_column::<MockColumn>("Name");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), MockColumn::Name);
    }

    #[test]
    fn test_invalid_column_name() {
        let result = get_column::<MockColumn>("invalid_field");
        assert!(result.is_err());
    }

    // Mock Column for testing helper functions
    #[derive(Debug, PartialEq)]
    enum MockColumn {
        Name,
        UserName,
        CreatedAt,
    }

    impl std::str::FromStr for MockColumn {
        type Err = String;

        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
            match s {
                "Name" => Ok(MockColumn::Name),
                "UserName" => Ok(MockColumn::UserName),
                "CreatedAt" => Ok(MockColumn::CreatedAt),
                _ => Err(format!("Unknown column: {}", s)),
            }
        }
    }
}
