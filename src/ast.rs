/// Abstract Syntax Tree for Google AIP-160 Filter expressions
use std::fmt;

/// Root filter expression
#[derive(Debug, Clone, PartialEq)]
pub struct Filter {
    pub expression: Expression,
}

/// Filter expression that can be a logical operation or a restriction
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    Restriction(Restriction),
    Sequence(Sequence),
}

/// A comparison restriction (e.g., field = value)
#[derive(Debug, Clone, PartialEq)]
pub struct Restriction {
    pub field: String,
    pub comparator: Comparator,
    pub value: Value,
}

/// Comparison operators
#[derive(Debug, Clone, PartialEq)]
pub enum Comparator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Has, // The ':' operator for substring/contains matching
}

/// Field sequence (for nested fields like user.name)
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub parts: Vec<String>,
}

/// Values that can appear in filter expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

impl Value {
    /// Check if this value is a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Check if this value is a number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Check if this value is a boolean
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

impl fmt::Display for Comparator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Comparator::Equal => write!(f, "="),
            Comparator::NotEqual => write!(f, "!="),
            Comparator::GreaterThan => write!(f, ">"),
            Comparator::GreaterThanOrEqual => write!(f, ">="),
            Comparator::LessThan => write!(f, "<"),
            Comparator::LessThanOrEqual => write!(f, "<="),
            Comparator::Has => write!(f, ":"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Number(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
        }
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::And(left, right) => write!(f, "({} AND {})", left, right),
            Expression::Or(left, right) => write!(f, "({} OR {})", left, right),
            Expression::Not(expr) => write!(f, "NOT {}", expr),
            Expression::Restriction(r) => write!(f, "{} {} {}", r.field, r.comparator, r.value),
            Expression::Sequence(s) => write!(f, "{}", s.parts.join(".")),
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.expression)
    }
}
