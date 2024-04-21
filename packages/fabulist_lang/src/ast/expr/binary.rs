use pest::iterators::Pair;

use crate::parser::Rule;

use super::{Error, Expr};

#[derive(Debug)]
pub enum BinaryOperator {
    Divide,
    Multiply,
    Addition,
    Subtraction,
    GreaterThan,
    GreaterEqual,
    LessThan,
    LessEqual,
    EqualEqual,
    NotEqual,
    And,
    Or,
}

#[derive(Debug)]
pub struct BinaryExpr {
    pub left: Expr,
    pub operator: Option<BinaryOperator>,
    pub right: Option<Expr>,
}

impl TryFrom<Pair<'_, Rule>> for BinaryExpr {
    type Error = Error;
    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let value_span = value.as_span();
        let inner = value.into_inner();

        let left = match inner.find_first_tagged("left") {
            Some(left) => Ok(Expr::try_from(left)?),
            None => Err(Error::map_span(value_span, "Expected a value expression")),
        }?;
        let operator = match inner.find_first_tagged("operator") {
            Some(operator) => {
                let operator_span = operator.as_span();
                Some(match operator.as_str() {
                    "/" => Ok(BinaryOperator::Divide),
                    "*" => Ok(BinaryOperator::Multiply),
                    "+" => Ok(BinaryOperator::Addition),
                    "-" => Ok(BinaryOperator::Subtraction),
                    ">" => Ok(BinaryOperator::GreaterThan),
                    ">=" => Ok(BinaryOperator::GreaterEqual),
                    "<" => Ok(BinaryOperator::LessThan),
                    "<=" => Ok(BinaryOperator::LessEqual),
                    "==" => Ok(BinaryOperator::EqualEqual),
                    "!=" => Ok(BinaryOperator::NotEqual),
                    "&&" => Ok(BinaryOperator::And),
                    "||" => Ok(BinaryOperator::Or),
                    _ => Err(Error::map_span(operator_span, "Invalid binary operator")),
                }?)
            }
            None => None,
        };
        let right = match inner.find_first_tagged("right") {
            Some(right) => Some(Expr::try_from(right)?),
            None => None,
        };

        Ok(BinaryExpr {
            left,
            operator,
            right,
        })
    }
}

#[cfg(test)]
mod binary_expr_tests {
    use crate::ast::ParserTestHelper;

    use super::*;

    #[test]
    fn parses_binary_expr() {
        let test_helper = ParserTestHelper::<BinaryExpr>::new(Rule::expression, "BinaryExpr");
        test_helper.assert_parse("5 + 2");
        test_helper.assert_parse("5/ 2");
        test_helper.assert_parse("5 *2");
        test_helper.assert_parse("5== 2");
    }
}
