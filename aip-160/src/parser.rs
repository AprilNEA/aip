/// Parser for Google AIP-160 Filter expressions
use pest::Parser;
use pest_derive::Parser;

use crate::ast::{Comparator, Expression, Filter, Restriction, Sequence, Value};
use crate::error::{FilterError, Result};

#[derive(Parser)]
#[grammar = "filter.pest"]
pub struct FilterParser;

/// Parse a filter string into an AST
pub fn parse_filter(input: &str) -> Result<Filter> {
    let pairs = FilterParser::parse(Rule::filter, input)
        .map_err(|e| FilterError::ParseError(e.to_string()))?;

    for pair in pairs {
        if pair.as_rule() == Rule::filter {
            for inner_pair in pair.into_inner() {
                if inner_pair.as_rule() == Rule::expression {
                    let expression = parse_expression(inner_pair)?;
                    return Ok(Filter { expression });
                }
            }
        }
    }

    Err(FilterError::ParseError("No expression found".to_string()))
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> Result<Expression> {
    let mut pairs = pair.into_inner();
    let mut expr = parse_term(pairs.next().unwrap())?;

    while let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::or_op => {
                let right = parse_term(pairs.next().unwrap())?;
                expr = Expression::Or(Box::new(expr), Box::new(right));
            }
            _ => {}
        }
    }

    Ok(expr)
}

fn parse_term(pair: pest::iterators::Pair<Rule>) -> Result<Expression> {
    let mut pairs = pair.into_inner();
    let mut expr = parse_factor(pairs.next().unwrap())?;

    while let Some(pair) = pairs.next() {
        match pair.as_rule() {
            Rule::and_op => {
                let right = parse_factor(pairs.next().unwrap())?;
                expr = Expression::And(Box::new(expr), Box::new(right));
            }
            _ => {}
        }
    }

    Ok(expr)
}

fn parse_factor(pair: pest::iterators::Pair<Rule>) -> Result<Expression> {
    let mut pairs = pair.into_inner();
    let first = pairs.next().unwrap();

    match first.as_rule() {
        Rule::not_op => {
            // After NOT, parse the next element (which should be restriction or expression)
            let next = pairs.next().unwrap();
            let inner_expr = match next.as_rule() {
                Rule::restriction => parse_restriction(next).map(Expression::Restriction)?,
                Rule::expression => parse_expression(next)?,
                _ => {
                    return Err(FilterError::ParseError(format!(
                        "Unexpected rule after NOT: {:?}",
                        next.as_rule()
                    )))
                }
            };
            Ok(Expression::Not(Box::new(inner_expr)))
        }
        Rule::restriction => parse_restriction(first).map(Expression::Restriction),
        Rule::sequence => parse_sequence(first).map(Expression::Sequence),
        Rule::expression => parse_expression(first),
        _ => Err(FilterError::ParseError(format!(
            "Unexpected rule in factor: {:?}",
            first.as_rule()
        ))),
    }
}

fn parse_restriction(pair: pest::iterators::Pair<Rule>) -> Result<Restriction> {
    let mut pairs = pair.into_inner();

    let comparable = pairs.next().unwrap();
    let field = parse_comparable(comparable)?;

    let comparator = pairs.next().unwrap();
    let comparator = parse_comparator(comparator)?;

    let arg = pairs.next().unwrap();
    let value = parse_arg(arg)?;

    Ok(Restriction {
        field,
        comparator,
        value,
    })
}

fn parse_comparable(pair: pest::iterators::Pair<Rule>) -> Result<String> {
    let member = pair.into_inner().next().unwrap();
    Ok(member.as_str().to_string())
}

fn parse_comparator(pair: pest::iterators::Pair<Rule>) -> Result<Comparator> {
    let comp = pair.as_str();
    match comp {
        "=" => Ok(Comparator::Equal),
        "!=" => Ok(Comparator::NotEqual),
        ">" => Ok(Comparator::GreaterThan),
        ">=" => Ok(Comparator::GreaterThanOrEqual),
        "<" => Ok(Comparator::LessThan),
        "<=" => Ok(Comparator::LessThanOrEqual),
        ":" => Ok(Comparator::Has),
        _ => Err(FilterError::ParseError(format!(
            "Unknown comparator: {}",
            comp
        ))),
    }
}

fn parse_arg(pair: pest::iterators::Pair<Rule>) -> Result<Value> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::string => {
            let string_inner = inner.into_inner().next().unwrap();
            Ok(Value::String(string_inner.as_str().to_string()))
        }
        Rule::number => {
            let num = inner
                .as_str()
                .parse::<f64>()
                .map_err(|e| FilterError::ParseError(format!("Invalid number: {}", e)))?;
            Ok(Value::Number(num))
        }
        Rule::boolean => {
            let bool_val = match inner.as_str().to_lowercase().as_str() {
                "true" => true,
                "false" => false,
                _ => {
                    return Err(FilterError::ParseError(format!(
                        "Invalid boolean: {}",
                        inner.as_str()
                    )))
                }
            };
            Ok(Value::Boolean(bool_val))
        }
        Rule::null_val => Ok(Value::Null),
        _ => Err(FilterError::ParseError(format!(
            "Unknown arg type: {:?}",
            inner.as_rule()
        ))),
    }
}

fn parse_sequence(pair: pest::iterators::Pair<Rule>) -> Result<Sequence> {
    let mut parts = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::member {
            parts.push(inner.as_str().to_string());
        }
    }
    Ok(Sequence { parts })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_equality() {
        let filter = parse_filter("name = \"foo\"").unwrap();
        println!("{:?}", filter);
    }

    #[test]
    fn test_and_expression() {
        let filter = parse_filter("age > 18 AND active = true").unwrap();
        println!("{:?}", filter);
    }

    #[test]
    fn test_complex_expression() {
        let filter =
            parse_filter("(status = \"active\" OR status = \"pending\") AND created > \"2024-01-01\"")
                .unwrap();
        println!("{:?}", filter);
    }

    #[test]
    fn test_not_expression() {
        let filter = parse_filter("NOT deleted = true").unwrap();
        println!("{:?}", filter);
    }

    #[test]
    fn test_has_operator() {
        let filter = parse_filter("tags : \"important\"").unwrap();
        println!("{:?}", filter);
    }
}
