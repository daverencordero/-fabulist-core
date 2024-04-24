use pest::{error::LineColLocation, iterators::Pair};

use crate::{ast::expr::Expr, parser::Rule};

use super::Error;

#[derive(Debug)]
pub struct ArgumentBodyDfn {
    pub lcol: LineColLocation,
    pub arguments: Option<Vec<Expr>>,
}

impl TryFrom<Pair<'_, Rule>> for ArgumentBodyDfn {
    type Error = Error;
    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let argument_body_dfn_lcol = LineColLocation::from(value.as_span());

        if let Some(arguments) = value
            .into_inner()
            .find(|pair| pair.as_rule() == Rule::arguments)
        {
            let arg_expr = arguments
                .into_inner()
                .map(|pair| Expr::try_from(pair))
                .collect::<Result<Vec<Expr>, Error>>()?;
            Ok(ArgumentBodyDfn {
                arguments: Some(arg_expr),
                lcol: argument_body_dfn_lcol,
            })
        } else {
            Ok(ArgumentBodyDfn {
                arguments: None,
                lcol: argument_body_dfn_lcol,
            })
        }
    }
}

#[cfg(test)]
mod argument_body_tests {
    use crate::ast::ParserTestHelper;

    use super::*;

    #[test]
    pub fn parses_argument_body() {
        let test_helper =
            ParserTestHelper::<ArgumentBodyDfn>::new(Rule::argument_body, "ArgumentBodyDfn");
        test_helper.assert_parse(r#"("string", 5, true)"#);
    }
}
