/// SeaORM v2 integration
#[cfg(feature = "sea-orm-v2")]
use sea_orm_v2::{
    ColumnTrait, Condition, EntityTrait, IntoSimpleExpr,
    sea_query::{Alias, ColumnType as SeaColumnType, Expr, ExprTrait, SimpleExpr},
};

pub trait FieldMapper {
    /// Map a field name to a SeaORM column expression and its type
    fn map_field(&self, field: &str) -> Result<(SimpleExpr, SeaColumnType)>;
}

// Default implementation for EntityTrait where Column implements FromStr
impl<T> FieldMapper for T
where
    T: EntityTrait,
    <T as EntityTrait>::Column: std::str::FromStr,
    <<T as EntityTrait>::Column as std::str::FromStr>::Err: std::fmt::Display,
{
    fn map_field(&self, field: &str) -> Result<(SimpleExpr, SeaColumnType)> {
        let col = get_column::<<T as EntityTrait>::Column>(field)?;
        #[allow(deprecated)] // implementation detail of sea-orm
        let col_type = col.def().get_column_type().clone();
        Ok((col.into_simple_expr(), col_type))
    }
}

use crate::ast::{Comparator, Expression, Filter, Restriction, Value};
use crate::error::{FilterError, Result};

/// Helper function to convert a field name to a column
///
/// Supports both exact matches and snake_case to PascalCase conversion.
#[cfg(feature = "sea-orm-v2")]
pub fn column_from_str<C>(field: &str) -> Result<SimpleExpr>
where
    C: std::str::FromStr + sea_orm_v2::IntoSimpleExpr,
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
/// use sea_orm_v2::entity::prelude::*;
///
/// let filter = parse_filter("name = \"Alice\" AND age > 18")?;
/// let condition = filter.to_condition::<Entity>()?;
///
/// Entity::find().filter(condition).all(db).await?;
/// ```
#[cfg(feature = "sea-orm-v2")]
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
    fn to_condition<M>(&self, mapper: &M) -> Result<Condition>
    where
        M: FieldMapper;
}

#[cfg(feature = "sea-orm-v2")]
impl ToSeaOrmCondition for Filter {
    fn to_condition<M>(&self, mapper: &M) -> Result<Condition>
    where
        M: FieldMapper,
    {
        expression_to_condition(mapper, &self.expression)
    }
}

#[cfg(feature = "sea-orm-v2")]
fn expression_to_condition<M>(mapper: &M, expr: &Expression) -> Result<Condition>
where
    M: FieldMapper,
{
    match expr {
        Expression::And(left, right) => {
            let left_cond = expression_to_condition(mapper, left)?;
            let right_cond = expression_to_condition(mapper, right)?;
            Ok(Condition::all().add(left_cond).add(right_cond))
        }
        Expression::Or(left, right) => {
            let left_cond = expression_to_condition(mapper, left)?;
            let right_cond = expression_to_condition(mapper, right)?;
            Ok(Condition::any().add(left_cond).add(right_cond))
        }
        Expression::Not(inner) => {
            let inner_cond = expression_to_condition(mapper, inner)?;
            Ok(inner_cond.not())
        }
        Expression::Restriction(restriction) => restriction_to_condition(mapper, restriction),
        Expression::Sequence(_) => Err(FilterError::UnsupportedOperation(
            "Sequences are not yet supported in SeaORM conversion".to_string(),
        )),
    }
}

/// Helper to get the Column instance from field name
#[cfg(feature = "sea-orm-v2")]
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

#[cfg(feature = "sea-orm-v2")]
fn restriction_to_condition<M>(mapper: &M, restriction: &Restriction) -> Result<Condition>
where
    M: FieldMapper,
{
    let (column_expr, col_type) = mapper.map_field(&restriction.field)?;
    let column = column_expr.clone();

    // Apply appropriate casting based on column type
    let value_expr: SimpleExpr = match (&restriction.value, col_type) {
        // UUID columns - cast string to uuid
        (Value::String(s), SeaColumnType::Uuid) => {
            Expr::val(s.as_str()).cast_as(Alias::new("uuid")).into()
        }
        // Timestamp columns - cast string to timestamp
        (Value::String(s), SeaColumnType::TimestampWithTimeZone) => Expr::val(s.as_str())
            .cast_as(Alias::new("timestamptz"))
            .into(),
        (Value::String(s), SeaColumnType::Timestamp) => Expr::val(s.as_str())
            .cast_as(Alias::new("timestamp"))
            .into(),
        // Date columns
        (Value::String(s), SeaColumnType::Date) => {
            Expr::val(s.as_str()).cast_as(Alias::new("date")).into()
        }
        // Time columns
        (Value::String(s), SeaColumnType::Time) => {
            Expr::val(s.as_str()).cast_as(Alias::new("time")).into()
        }
        // Enum columns - cast string to the enum type
        (Value::String(s), SeaColumnType::Enum { name, .. }) => Expr::val(s.as_str())
            .cast_as(Alias::new(name.to_string()))
            .into(),
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
        (Value::Null, _) => Expr::value(sea_orm_v2::sea_query::Value::String(None)).into(),
    };

    let condition = match (&restriction.comparator, &restriction.value) {
        (Comparator::Equal, Value::Null) => Condition::all().add(column.is_null()),
        (Comparator::NotEqual, Value::Null) => Condition::all().add(column.is_not_null()),
        (Comparator::Has, Value::String(s)) => {
            Condition::all().add(column.like(format!("%{}%", s)))
        }
        (Comparator::Has, _) => {
            return Err(FilterError::UnsupportedOperation(
                "Has operator (:) requires a string value".to_string(),
            ));
        }
        (Comparator::Equal, _) => Condition::all().add(Expr::expr(column).eq(value_expr)),
        (Comparator::NotEqual, _) => Condition::all().add(Expr::expr(column).ne(value_expr)),
        (Comparator::GreaterThan, _) => Condition::all().add(Expr::expr(column).gt(value_expr)),
        (Comparator::GreaterThanOrEqual, _) => {
            Condition::all().add(Expr::expr(column).gte(value_expr))
        }
        (Comparator::LessThan, _) => Condition::all().add(Expr::expr(column).lt(value_expr)),
        (Comparator::LessThanOrEqual, _) => {
            Condition::all().add(Expr::expr(column).lte(value_expr))
        }
    };

    Ok(condition)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Unit tests here are minimal because full Entity integration
    // requires complex trait implementations. See tests/sea_orm_v2_integration.rs
    // for comprehensive integration tests with a real SeaORM v2 Entity.

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
