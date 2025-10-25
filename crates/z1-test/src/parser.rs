use crate::ast::*;
use crate::lexer::{lex_test, TestToken, TestTokenKind};
use thiserror::Error;
use z1_ast::{Block, Span, TypeExpr};

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, got {got} at position {pos}")]
    UnexpectedToken {
        expected: String,
        got: String,
        pos: u32,
    },
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Invalid syntax: {message}")]
    InvalidSyntax { message: String },
}

pub struct Parser {
    tokens: Vec<TestToken>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<TestToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> &TestToken {
        self.tokens
            .get(self.pos)
            .unwrap_or(&self.tokens[self.tokens.len() - 1])
    }

    fn peek(&self) -> TestTokenKind {
        self.current().kind
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
    }

    fn expect(&mut self, kind: TestTokenKind) -> Result<TestToken, ParseError> {
        if self.peek() == kind {
            let token = self.current().clone();
            self.advance();
            Ok(token)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{kind:?}"),
                got: format!("{:?}", self.peek()),
                pos: self.current().span.start,
            })
        }
    }

    fn match_token(&mut self, kind: TestTokenKind) -> bool {
        if self.peek() == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn parse_test_file(&mut self) -> Result<TestFile, ParseError> {
        let start_span = self.current().span;
        let mut test_file = TestFile::new();

        // Parse config if present
        if self.peek() == TestTokenKind::KwConfig {
            test_file.config = self.parse_config()?;
        }

        // Parse items
        while self.peek() != TestTokenKind::Eof {
            match self.peek() {
                TestTokenKind::KwSpec => {
                    test_file.specs.push(self.parse_spec()?);
                }
                TestTokenKind::KwProp => {
                    test_file.props.push(self.parse_prop()?);
                }
                TestTokenKind::KwFixture => {
                    test_file.fixtures.push(self.parse_fixture()?);
                }
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: format!("Unexpected token: {:?}", self.peek()),
                    });
                }
            }
        }

        let end_span = self.current().span;
        test_file.span = Span::new(start_span.start, end_span.end);

        Ok(test_file)
    }

    fn parse_config(&mut self) -> Result<TestConfig, ParseError> {
        self.expect(TestTokenKind::KwConfig)?;
        self.expect(TestTokenKind::LBrace)?;

        let mut config = TestConfig::default();

        while self.peek() != TestTokenKind::RBrace {
            // Accept either Ident or keyword tokens for config keys
            let key = match self.peek() {
                TestTokenKind::Ident | TestTokenKind::KwTimeout | TestTokenKind::KwSeed => {
                    let tok = self.current().clone();
                    self.advance();
                    tok
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "config key".to_string(),
                        got: format!("{:?}", self.peek()),
                        pos: self.current().span.start,
                    })
                }
            };
            self.expect(TestTokenKind::Colon)?;

            match key.lexeme.as_str() {
                "timeout_ms" => {
                    let value = self.expect(TestTokenKind::Number)?;
                    config.timeout_ms = Some(value.lexeme.parse().unwrap_or(0));
                }
                "parallel" => {
                    let value = self.expect(TestTokenKind::Number)?;
                    config.parallel = Some(value.lexeme.parse().unwrap_or(1));
                }
                "seed" => {
                    let value = self.expect(TestTokenKind::Number)?;
                    config.seed = Some(value.lexeme.parse().unwrap_or(0));
                }
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: format!("Unknown config key: {}", key.lexeme),
                    });
                }
            }

            // Optional comma or semicolon
            self.match_token(TestTokenKind::Comma);
            self.match_token(TestTokenKind::Semi);
        }

        self.expect(TestTokenKind::RBrace)?;
        Ok(config)
    }

    fn parse_spec(&mut self) -> Result<Spec, ParseError> {
        let start = self.current().span;
        self.expect(TestTokenKind::KwSpec)?;

        let name_token = self.expect(TestTokenKind::String)?;
        let name = name_token.lexeme.trim_matches('"').to_string();

        let attrs = self.parse_attrs()?;
        let body = self.parse_block()?;

        let end = self.current().span;

        Ok(Spec {
            name,
            attrs,
            body,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_prop(&mut self) -> Result<Prop, ParseError> {
        let start = self.current().span;
        self.expect(TestTokenKind::KwProp)?;

        let name_token = self.expect(TestTokenKind::String)?;
        let name = name_token.lexeme.trim_matches('"').to_string();

        let attrs = self.parse_attrs()?;

        self.expect(TestTokenKind::KwForAll)?;
        self.expect(TestTokenKind::LParen)?;

        let mut bindings = Vec::new();
        loop {
            if self.peek() == TestTokenKind::RParen {
                break;
            }

            let binding = self.parse_gen_binding()?;
            bindings.push(binding);

            if !self.match_token(TestTokenKind::Comma) {
                break;
            }
        }

        self.expect(TestTokenKind::RParen)?;

        let mut runs = 100;
        let mut seed = 0;

        if self.match_token(TestTokenKind::KwRuns) {
            let runs_token = self.expect(TestTokenKind::Number)?;
            runs = runs_token.lexeme.parse().unwrap_or(100);
        }

        if self.match_token(TestTokenKind::KwSeed) {
            let seed_token = self.expect(TestTokenKind::Number)?;
            seed = seed_token.lexeme.parse().unwrap_or(0);
        }

        let body = self.parse_block()?;
        let end = self.current().span;

        Ok(Prop {
            name,
            attrs,
            bindings,
            runs,
            seed,
            body,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_gen_binding(&mut self) -> Result<GenBinding, ParseError> {
        let start = self.current().span;
        let name = self.expect(TestTokenKind::Ident)?;
        self.expect(TestTokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        let end = self.current().span;

        Ok(GenBinding {
            name: name.lexeme,
            ty,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        // Simplified type parser - just handles identifiers for MVP
        let token = self.expect(TestTokenKind::Ident)?;
        Ok(TypeExpr::Path(vec![token.lexeme]))
    }

    fn parse_fixture(&mut self) -> Result<Fixture, ParseError> {
        let start = self.current().span;
        self.expect(TestTokenKind::KwFixture)?;

        let name = self.expect(TestTokenKind::Ident)?;

        let ty = if self.match_token(TestTokenKind::Colon) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };

        self.expect(TestTokenKind::Eq)?;
        let body = self.parse_block()?;
        self.expect(TestTokenKind::Semi)?;

        let end = self.current().span;

        Ok(Fixture {
            name: name.lexeme,
            ty,
            body,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_attrs(&mut self) -> Result<TestAttrs, ParseError> {
        let mut attrs = TestAttrs::default();

        if !self.match_token(TestTokenKind::KwWith) {
            return Ok(attrs);
        }

        self.expect(TestTokenKind::LBrace)?;

        while self.peek() != TestTokenKind::RBrace {
            // Accept either Ident or keyword tokens for attribute names
            let key = match self.peek() {
                TestTokenKind::Ident
                | TestTokenKind::KwTimeout
                | TestTokenKind::KwSkip
                | TestTokenKind::KwOnly
                | TestTokenKind::KwTags => {
                    let tok = self.current().clone();
                    self.advance();
                    tok
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        expected: "attribute name".to_string(),
                        got: format!("{:?}", self.peek()),
                        pos: self.current().span.start,
                    })
                }
            };
            self.expect(TestTokenKind::Colon)?;

            match key.lexeme.as_str() {
                "timeout" | "timeout_ms" => {
                    let value = self.expect(TestTokenKind::Number)?;
                    attrs.timeout_ms = Some(value.lexeme.parse().unwrap_or(0));
                }
                "skip" => {
                    let value = self.expect(TestTokenKind::Ident)?;
                    attrs.skip = value.lexeme == "true";
                }
                "only" => {
                    let value = self.expect(TestTokenKind::Ident)?;
                    attrs.only = value.lexeme == "true";
                }
                "tags" => {
                    self.expect(TestTokenKind::LBracket)?;
                    while self.peek() != TestTokenKind::RBracket {
                        let tag = self.expect(TestTokenKind::String)?;
                        attrs.tags.push(tag.lexeme.trim_matches('"').to_string());
                        self.match_token(TestTokenKind::Comma);
                    }
                    self.expect(TestTokenKind::RBracket)?;
                }
                _ => {
                    return Err(ParseError::InvalidSyntax {
                        message: format!("Unknown attribute: {}", key.lexeme),
                    });
                }
            }

            self.match_token(TestTokenKind::Comma);
        }

        self.expect(TestTokenKind::RBrace)?;
        Ok(attrs)
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.current().span;
        self.expect(TestTokenKind::LBrace)?;

        let mut content = String::new();
        let mut depth = 1;

        while depth > 0 && self.peek() != TestTokenKind::Eof {
            let token = self.current();
            match token.kind {
                TestTokenKind::LBrace => {
                    depth += 1;
                    content.push_str(&token.lexeme);
                    content.push(' ');
                }
                TestTokenKind::RBrace => {
                    depth -= 1;
                    if depth > 0 {
                        content.push_str(&token.lexeme);
                        content.push(' ');
                    }
                }
                _ => {
                    content.push_str(&token.lexeme);
                    content.push(' ');
                }
            }
            self.advance();
        }

        let end = self.current().span;

        Ok(Block {
            raw: content.trim().to_string(),
            statements: Vec::new(),
            span: Span::new(start.start, end.end),
        })
    }
}

pub fn parse_test_file(source: &str) -> Result<TestFile, ParseError> {
    let tokens = lex_test(source);
    let mut parser = Parser::new(tokens);
    parser.parse_test_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_file() {
        let result = parse_test_file("");
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.specs.len(), 0);
        assert_eq!(file.props.len(), 0);
    }

    #[test]
    fn parse_simple_spec() {
        let input = r#"spec "test name" { assert 1 + 1 == 2; }"#;
        let result = parse_test_file(input);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.specs.len(), 1);
        assert_eq!(file.specs[0].name, "test name");
    }

    #[test]
    fn parse_spec_with_attributes() {
        let input = r#"spec "test" with { timeout: 5000, skip: false } { }"#;
        let result = parse_test_file(input);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.specs[0].attrs.timeout_ms, Some(5000));
        assert!(!file.specs[0].attrs.skip);
    }

    #[test]
    fn parse_config() {
        let input = "config { timeout_ms: 3000 }";
        let result = parse_test_file(input);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.config.timeout_ms, Some(3000));
    }

    #[test]
    fn parse_property_test() {
        let input = r#"prop "commutative" for_all (a: U32, b: U32) runs 100 seed 42 { }"#;
        let result = parse_test_file(input);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.props.len(), 1);
        assert_eq!(file.props[0].name, "commutative");
        assert_eq!(file.props[0].bindings.len(), 2);
        assert_eq!(file.props[0].runs, 100);
        assert_eq!(file.props[0].seed, 42);
    }

    #[test]
    fn parse_fixture() {
        let input = "fixture x: U32 = { 42 };";
        let result = parse_test_file(input);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.fixtures.len(), 1);
        assert_eq!(file.fixtures[0].name, "x");
    }

    #[test]
    fn parse_multiple_specs() {
        let input = r#"
            spec "test1" { }
            spec "test2" { }
            spec "test3" { }
        "#;
        let result = parse_test_file(input);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.specs.len(), 3);
    }

    #[test]
    fn parse_mixed_file() {
        let input = r#"
            config { timeout_ms: 5000 }
            fixture base: U32 = { 10 };
            spec "unit test" { }
            prop "property" for_all (x: U32) runs 50 { }
        "#;
        let result = parse_test_file(input);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.config.timeout_ms, Some(5000));
        assert_eq!(file.fixtures.len(), 1);
        assert_eq!(file.specs.len(), 1);
        assert_eq!(file.props.len(), 1);
    }

    #[test]
    fn reject_invalid_syntax() {
        let input = "invalid syntax here";
        let result = parse_test_file(input);
        assert!(result.is_err());
    }
}
