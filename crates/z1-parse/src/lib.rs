use thiserror::Error;
use z1_ast::{Import, Item, Module, ModulePath, Span};
use z1_lex::{lex, Token, TokenKind};

pub fn parse_module(source: &str) -> Result<Module, ParseError> {
    let tokens = lex(source);
    Parser::new(tokens).parse()
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unexpected token: expected {expected}, found {found:?}")]
    Unexpected {
        expected: &'static str,
        found: TokenKind,
        span: Span,
    },
    #[error("invalid literal: {message}")]
    Invalid { message: String, span: Span },
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn parse(mut self) -> Result<Module, ParseError> {
        let start_span = self.peek().span;
        self.expect(TokenKind::KwModule, "module keyword")?;
        let path_token = self.expect(TokenKind::Ident, "module path")?;
        let path = ModulePath::from_parts(
            path_token
                .lexeme
                .split('.')
                .map(|part| part.to_string())
                .collect(),
        );
        let version = self.parse_version()?;
        let mut ctx_budget = None;
        let mut caps = Vec::new();

        loop {
            match self.peek().kind {
                TokenKind::KwCtx => ctx_budget = Some(self.parse_ctx_budget()?),
                TokenKind::KwCaps => caps = self.parse_caps()?,
                _ => break,
            }
        }

        let mut items = Vec::new();
        while !self.at(TokenKind::Eof) {
            if self.peek().kind == TokenKind::KwUse {
                let import = self.parse_import()?;
                items.push(Item::Import(import));
            } else {
                // Skip tokens we don't understand yet.
                self.advance();
            }
        }

        let span = Span::new(
            start_span.start,
            self.tokens[self.pos.saturating_sub(1)].span.end,
        );
        Ok(Module::new(path, version, ctx_budget, caps, items, span))
    }

    fn parse_version(&mut self) -> Result<Option<String>, ParseError> {
        if !self.at(TokenKind::Colon) {
            return Ok(None);
        }
        self.advance();
        let mut parts = Vec::new();
        let number = self.expect(TokenKind::Number, "version number")?;
        parts.push(number.lexeme.clone());
        while self.at(TokenKind::Dot) {
            self.advance();
            let segment = self.expect(TokenKind::Number, "version segment")?;
            parts.push(segment.lexeme.clone());
        }
        Ok(Some(parts.join(".")))
    }

    fn parse_ctx_budget(&mut self) -> Result<u32, ParseError> {
        self.expect(TokenKind::KwCtx, "ctx keyword")?;
        self.expect(TokenKind::Eq, "equals after ctx")?;
        let number = self.expect(TokenKind::Number, "ctx integer")?;
        number.lexeme.parse().map_err(|_| ParseError::Invalid {
            message: "ctx must be an integer".into(),
            span: number.span,
        })
    }

    fn parse_caps(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect(TokenKind::KwCaps, "caps keyword")?;
        self.expect(TokenKind::Eq, "equals after caps")?;
        self.expect(TokenKind::LBracket, "opening bracket")?;
        let mut caps = Vec::new();
        while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
            let cap = self.expect(TokenKind::Ident, "capability name")?;
            caps.push(cap.lexeme.clone());
            if self.at(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RBracket, "] after caps list")?;
        Ok(caps)
    }

    fn parse_import(&mut self) -> Result<Import, ParseError> {
        let start = self.expect(TokenKind::KwUse, "use keyword")?.span;
        let path_token = self.expect(TokenKind::String, "string import path")?;
        let alias = if self.at(TokenKind::KwAs) {
            self.advance();
            Some(
                self.expect(TokenKind::Ident, "alias identifier")?
                    .lexeme
                    .clone(),
            )
        } else {
            None
        };
        let only = if self.at(TokenKind::KwOnly) {
            self.advance();
            self.expect(TokenKind::LBracket, "opening bracket after only")?;
            let mut list = Vec::new();
            while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
                let item = self.expect(TokenKind::Ident, "only identifier")?;
                list.push(item.lexeme.clone());
                if self.at(TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.expect(TokenKind::RBracket, "] after only list")?;
            list
        } else {
            Vec::new()
        };

        // optional trailing semicolons.
        if self.at(TokenKind::Semi) {
            self.advance();
        }

        Ok(Import {
            path: strip_quotes(&path_token.lexeme),
            alias,
            only,
            span: Span::new(start.start, self.previous().span.end),
        })
    }

    fn expect(&mut self, kind: TokenKind, expected: &'static str) -> Result<Token, ParseError> {
        if self.peek().kind == kind {
            Ok(self.advance())
        } else {
            Err(ParseError::Unexpected {
                expected,
                found: self.peek().kind,
                span: self.peek().span,
            })
        }
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos = usize::min(self.pos + 1, self.tokens.len() - 1);
        token
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.pos.saturating_sub(1)]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }
}

fn strip_quotes(input: &str) -> String {
    input
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(input)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sample_cell() {
        let source = include_str!("../../../fixtures/cells/http_server.z1c");
        let module = parse_module(source).expect("module parses");
        let segments = module.path.as_str_vec().to_vec();
        assert_eq!(segments, vec!["http".to_string(), "server".to_string()]);
        assert_eq!(module.version.as_deref(), Some("1.0"));
        assert_eq!(module.ctx_budget, Some(128));
        assert_eq!(module.caps, vec!["net"]);
        assert_eq!(module.items.len(), 1);
        match &module.items[0] {
            Item::Import(import) => {
                assert_eq!(import.path, "std/http");
                assert_eq!(import.alias.as_deref(), Some("H"));
                assert_eq!(import.only, vec!["listen", "Req", "Res"]);
            }
        }
    }
}
