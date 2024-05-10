use fabulist_derive::SyntaxTree;
use pest::{error::LineColLocation, iterators::Pair};

use crate::{error::Error, parser::Rule};

use super::expr::{primitive::Primitive, Expr};

#[derive(Debug, Clone)]
pub enum ElseClause {
    If(IfStmt),
    Block(BlockStmt),
}

impl TryFrom<Pair<'_, Rule>> for ElseClause {
    type Error = Error;
    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let value_span = value.as_span();
        let mut inner = value.into_inner();

        if let Some(if_stmt) = inner.clone().find(|pair| pair.as_rule() == Rule::if_stmt) {
            Ok(ElseClause::If(IfStmt::try_from(if_stmt)?))
        } else if let Some(block_stmt) = inner.find(|pair| pair.as_rule() == Rule::block_stmt) {
            Ok(ElseClause::Block(BlockStmt::try_from(block_stmt)?))
        } else {
            Err(Error::map_span(
                value_span,
                "Expected an `if` or `block` statement",
            ))
        }
    }
}

#[derive(SyntaxTree, Debug, Clone)]
pub enum Stmt {
    #[production(statements: Vec<Stmt>)]
    Block(Box<BlockStmt>),
    #[production(condition: Expr, block_stmt: BlockStmt, else_stmt: Option<Box<ElseClause>>)]
    If(Box<IfStmt>),
    #[production(identifier: Primitive, value: Expr)]
    Let(Box<LetStmt>),
    #[production(path: Primitive)]
    Goto(Box<GotoStmt>),
}

impl From<BlockStmt> for Stmt {
    fn from(value: BlockStmt) -> Self {
        Self::Block(Box::new(value))
    }
}

impl From<IfStmt> for Stmt {
    fn from(value: IfStmt) -> Self {
        Self::If(Box::new(value))
    }
}

impl From<LetStmt> for Stmt {
    fn from(value: LetStmt) -> Self {
        Self::Let(Box::new(value))
    }
}

impl From<GotoStmt> for Stmt {
    fn from(value: GotoStmt) -> Self {
        Self::Goto(Box::new(value))
    }
}

impl TryFrom<Pair<'_, Rule>> for Stmt {
    type Error = Error;
    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let stmt_span = value.as_span();
        match value.as_rule() {
            Rule::statement => match value.into_inner().next() {
                Some(inner) => Ok(Stmt::try_from(inner)?),
                None => unreachable!(),
            },
            Rule::block_stmt => Ok(BlockStmt::try_from(value)?.into()),
            Rule::if_stmt => Ok(IfStmt::try_from(value)?.into()),
            Rule::let_stmt => Ok(LetStmt::try_from(value)?.into()),
            Rule::goto_stmt => Ok(GotoStmt::try_from(value)?.into()),
            _ => Err(Error::map_span(stmt_span, "Invalid statement")),
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for BlockStmt {
    type Error = Error;
    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let value_lcol = LineColLocation::from(value.as_span());
        let statements = value
            .into_inner()
            .map(Stmt::try_from)
            .collect::<Result<Vec<Stmt>, Error>>()?;

        Ok(BlockStmt {
            statements,
            lcol: value_lcol,
        })
    }
}

impl TryFrom<Pair<'_, Rule>> for IfStmt {
    type Error = Error;
    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let value_span = value.as_span();
        let value_lcol = LineColLocation::from(value_span);
        let mut inner = value.into_inner();

        let condition = match inner.find(|pair| pair.as_node_tag() == Some("condition")) {
            Some(condition) => Expr::try_from(condition),
            None => Err(Error::map_span(value_span, "Expected condition expression")),
        }?;
        let block_stmt = match inner.find(|pair| pair.as_rule() == Rule::block_stmt) {
            Some(block_stmt) => BlockStmt::try_from(block_stmt),
            None => Err(Error::map_span(value_span, "Expected block statement")),
        }?;
        let else_stmt = match inner.find(|pair| pair.as_rule() == Rule::else_stmt) {
            Some(else_stmt) => Some(Box::new(ElseClause::try_from(else_stmt)?)),
            None => None,
        };

        Ok(IfStmt {
            condition,
            block_stmt,
            else_stmt,
            lcol: value_lcol,
        })
    }
}

impl TryFrom<Pair<'_, Rule>> for LetStmt {
    type Error = Error;
    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let value_span = value.as_span();
        let value_lcol = LineColLocation::from(value_span);
        let mut inner = value.into_inner();

        let identifier = match inner.find(|pair| pair.as_rule() == Rule::identifier) {
            Some(identifier) => Primitive::try_from(identifier),
            None => Err(Error::map_span(value_span, "Expected an identifier")),
        }?;
        let value = match inner.find(|pair| pair.as_node_tag() == Some("value")) {
            Some(expression) => Expr::try_from(expression),
            None => Err(Error::map_span(value_span, "Expected value expression")),
        }?;

        Ok(LetStmt {
            identifier,
            value,
            lcol: value_lcol,
        })
    }
}

impl TryFrom<Pair<'_, Rule>> for GotoStmt {
    type Error = Error;
    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let value_span = value.as_span();
        let value_lcol = LineColLocation::from(value_span);

        let path = match value.into_inner().find(|pair| pair.as_rule() == Rule::path) {
            Some(path) => Primitive::try_from(path),
            None => Err(Error::map_span(value_span, "Expected path expression")),
        }?;

        Ok(GotoStmt {
            path,
            lcol: value_lcol,
        })
    }
}

#[cfg(test)]
mod stmt_tests {
    use crate::ast::ParserTestHelper;

    use super::*;

    #[test]
    fn parses_block_stmt() {
        let test_helper = ParserTestHelper::<BlockStmt>::new(Rule::block_stmt, "BlockStmt");
        test_helper.assert_parse(
            r#"{
                let key = "value";
                goto module_1::part_1;
                if true {} else if true {} else {}
            }"#,
        );
    }

    #[test]
    fn parses_if_stmt() {
        let test_helper = ParserTestHelper::<IfStmt>::new(Rule::if_stmt, "IfStmt");
        test_helper.assert_parse("if true {}");
        test_helper.assert_parse("if true {} else {}");
    }

    #[test]
    fn parses_let_stmt() {
        let test_helper = ParserTestHelper::<LetStmt>::new(Rule::let_stmt, "LetStmt");
        test_helper.assert_parse("let key = \"value\";");
    }

    #[test]
    fn parses_goto_stmt() {
        let test_helper = ParserTestHelper::<GotoStmt>::new(Rule::goto_stmt, "GotoStmt");
        test_helper.assert_parse("goto module_1::part_1;");
    }
}
