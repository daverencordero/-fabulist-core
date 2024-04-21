use pest::iterators::Pair;

use crate::{ast::dfn::object::Object, parser::Rule};

use super::Error;

#[derive(Debug)]
pub struct QuoteDecl {
    pub text: String,
    pub properties: Option<Object>,
}

impl TryFrom<Pair<'_, Rule>> for QuoteDecl {
    type Error = Error;
    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        let value_span = value.as_span();
        let mut inner = value.into_inner();

        let text = match inner.find_first_tagged("text") {
            Some(text) => Ok(match text.into_inner().next() {
                Some(text) => Ok(text.as_str().to_string()),
                None => Err(Error::map_span(value_span, "Expected string value")),
            }?),
            None => Err(Error::map_span(value_span, "Expected text expression")),
        }?;

        let properties = match inner.find(|pair| pair.as_rule() == Rule::object) {
            Some(object) => Some(Object::try_from(object)?),
            None => None,
        };

        Ok(QuoteDecl { text, properties })
    }
}

#[cfg(test)]
mod quote_elem_tests {
    use crate::ast::ParserTestHelper;

    use super::*;

    #[test]
    fn parses_quote_elem() {
        let test_helper = ParserTestHelper::<QuoteDecl>::new(Rule::quote_decl, "QuoteDecl");
        test_helper.assert_parse(r#"> "I'm an example quote""#);

        let test_helper = ParserTestHelper::<QuoteDecl>::new(Rule::narration_decl, "QuoteDecl");
        test_helper.assert_parse(r#"* "I'm an example narration""#);

        let test_helper = ParserTestHelper::<QuoteDecl>::new(Rule::choice_decl, "QuoteDecl");
        test_helper.assert_parse(r#"- "I'm an example choice""#);
    }
}
