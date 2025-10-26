use thiserror::Error;
use z1_ast::{
    Block, FnDecl, Import, Item, Module, ModulePath, Param, RecordField, Span, SymbolMap,
    SymbolPair, TypeDecl, TypeExpr,
};
use z1_fmt::SymbolTable;
use z1_lex::{lex, Token, TokenKind};

pub fn parse_module(source: &str) -> Result<Module, ParseError> {
    let tokens = lex(source);
    Parser::new(source, tokens).parse()
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

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    symtable: SymbolTable,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str, tokens: Vec<Token>) -> Self {
        Self {
            source,
            tokens,
            pos: 0,
            symtable: SymbolTable::from_symbol_map(&SymbolMap::default()),
        }
    }

    /// Normalize an identifier to its canonical long form using the symbol table
    fn normalize_ident(&self, ident: &str) -> String {
        self.symtable.normalize_ident(ident)
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

        // CRITICAL: Parse symbol map FIRST and build the symbol table
        // This must happen before any declarations so we can normalize identifiers
        let mut symbol_map_item: Option<SymbolMap> = None;
        let saved_pos = self.pos;

        // Scan for symbol map
        while !self.at(TokenKind::Eof) {
            if self.at(TokenKind::Sym) {
                symbol_map_item = Some(self.parse_symbol_map()?);
                break;
            } else if matches!(self.peek().kind, TokenKind::KwType | TokenKind::KwFn) {
                // Stop if we hit a declaration before finding symbol map
                break;
            } else {
                self.advance();
            }
        }

        // Reset position and build symbol table
        self.pos = saved_pos;
        if let Some(ref sym_map) = symbol_map_item {
            self.symtable = SymbolTable::from_symbol_map(sym_map);
        }

        // Now parse all items (including symbol map again, which is OK)
        let mut items = Vec::new();
        while !self.at(TokenKind::Eof) {
            match self.peek().kind {
                TokenKind::KwUse => {
                    let import = self.parse_import()?;
                    items.push(Item::Import(import));
                }
                TokenKind::Sym => {
                    let sym = self.parse_symbol_map()?;
                    items.push(Item::Symbol(sym));
                }
                TokenKind::KwType => {
                    let ty = self.parse_type_decl()?;
                    items.push(Item::Type(ty));
                }
                TokenKind::KwFn => {
                    let func = self.parse_fn_decl()?;
                    items.push(Item::Fn(func));
                }
                TokenKind::Semi => {
                    self.advance();
                }
                _ => {
                    // Skip tokens we don't understand yet to avoid infinite loops.
                    self.advance();
                }
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
            let alias_token = self.expect(TokenKind::Ident, "alias identifier")?;
            // Normalize alias to long form
            Some(self.normalize_ident(&alias_token.lexeme))
        } else {
            None
        };
        let only = if self.at(TokenKind::KwOnly) {
            self.advance();
            self.expect(TokenKind::LBracket, "opening bracket after only")?;
            let mut list = Vec::new();
            while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
                let item = self.expect(TokenKind::Ident, "only identifier")?;
                // Normalize each imported item to long form
                list.push(self.normalize_ident(&item.lexeme));
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

    fn parse_symbol_map(&mut self) -> Result<SymbolMap, ParseError> {
        let start = self.expect(TokenKind::Sym, "#sym directive")?.span;
        self.expect(TokenKind::LBrace, "opening { in symbol map")?;
        let mut pairs = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let long = self.expect_ident_or_keyword("long identifier")?;
            self.expect(TokenKind::Colon, ": between long/short identifiers")?;
            let short = self.expect_ident_or_keyword("short identifier")?;
            let span = Span::new(long.span.start, short.span.end);
            pairs.push(SymbolPair {
                long: long.lexeme,
                short: short.lexeme,
                span,
            });
            if self.at(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        let end = self
            .expect(TokenKind::RBrace, "closing } in symbol map")?
            .span;
        Ok(SymbolMap {
            pairs,
            span: Span::new(start.start, end.end),
        })
    }

    fn parse_type_decl(&mut self) -> Result<TypeDecl, ParseError> {
        let start = self.expect(TokenKind::KwType, "type keyword")?.span;
        let name = self.expect_ident_or_keyword("type name")?;
        self.expect(TokenKind::Eq, "equals in type declaration")?;
        let expr = self.parse_type_expr()?;
        if self.at(TokenKind::Semi) {
            self.advance();
        }
        let end_span = self.previous().span;
        Ok(TypeDecl {
            name: self.normalize_ident(&name.lexeme), // Normalize to long form
            expr,
            span: Span::new(start.start, end_span.end),
        })
    }

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        match self.peek().kind {
            TokenKind::LBrace => self.parse_record_type(),
            TokenKind::Ident => self.parse_path_type(),
            _ => Err(ParseError::Unexpected {
                expected: "type expression",
                found: self.peek().kind,
                span: self.peek().span,
            }),
        }
    }

    fn parse_path_type(&mut self) -> Result<TypeExpr, ParseError> {
        let ident = self.expect(TokenKind::Ident, "type identifier")?;
        let mut segments = vec![self.normalize_ident(&ident.lexeme)]; // Normalize
        while self.at(TokenKind::Dot) {
            self.advance();
            let segment = self.expect(TokenKind::Ident, "path segment")?;
            segments.push(self.normalize_ident(&segment.lexeme)); // Normalize
        }
        Ok(TypeExpr::Path(segments))
    }

    fn parse_record_type(&mut self) -> Result<TypeExpr, ParseError> {
        self.expect(TokenKind::LBrace, "opening { in record type")?;
        let mut fields = Vec::new();
        while !self.at(TokenKind::RBrace) && !self.at(TokenKind::Eof) {
            let name = self.expect(TokenKind::Ident, "record field name")?;
            self.expect(TokenKind::Colon, ": in record field")?;
            let ty = self.parse_type_expr()?;
            let field_span = Span::new(name.span.start, self.previous().span.end);
            fields.push(RecordField {
                name: self.normalize_ident(&name.lexeme), // Normalize field name
                ty: Box::new(ty),
                span: field_span,
            });
            if self.at(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RBrace, "closing } in record type")?;
        Ok(TypeExpr::Record(fields))
    }

    fn parse_fn_decl(&mut self) -> Result<FnDecl, ParseError> {
        let start = self.expect(TokenKind::KwFn, "fn keyword")?.span;
        let name = self.expect_ident_or_keyword("function name")?;
        self.expect(TokenKind::LParen, "opening ( in parameter list")?;
        let params = self.parse_params()?;
        self.expect(TokenKind::RParen, "closing ) in parameter list")?;
        self.expect(TokenKind::Arrow, "-> return type")?;
        let ret = self.parse_type_expr()?;
        let effects = if self.at(TokenKind::KwEff) {
            self.parse_effects()?
        } else {
            Vec::new()
        };
        let body = self.parse_block()?;
        Ok(FnDecl {
            name: self.normalize_ident(&name.lexeme), // CRITICAL: Normalize function name
            params,
            ret,
            effects,
            span: Span::new(start.start, body.span.end),
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        while !self.at(TokenKind::RParen) && !self.at(TokenKind::Eof) {
            let name = if self.at(TokenKind::RParen) {
                break;
            } else {
                self.expect_ident_or_keyword("parameter name")?
            };
            self.expect(TokenKind::Colon, ": after parameter name")?;
            let ty = self.parse_type_expr()?;
            let span = Span::new(name.span.start, self.previous().span.end);
            params.push(Param {
                name: self.normalize_ident(&name.lexeme), // Normalize parameter name
                ty,
                span,
            });
            if self.at(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(params)
    }

    fn parse_effects(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect(TokenKind::KwEff, "eff keyword")?;
        self.expect(TokenKind::LBracket, "opening [ in effect list")?;
        let mut effects = Vec::new();
        while !self.at(TokenKind::RBracket) && !self.at(TokenKind::Eof) {
            let effect = self.expect(TokenKind::Ident, "effect identifier")?;
            effects.push(effect.lexeme.clone());
            if self.at(TokenKind::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        self.expect(TokenKind::RBracket, "closing ] in effect list")?;
        Ok(effects)
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let open = self.expect(TokenKind::LBrace, "opening { in block")?;
        let mut depth = 1;
        let mut end_span = open.span;
        while depth > 0 {
            let token = self.advance();
            match token.kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => {
                    depth -= 1;
                    end_span = token.span;
                }
                TokenKind::Eof => {
                    return Err(ParseError::Invalid {
                        message: "unterminated block".into(),
                        span: open.span,
                    })
                }
                _ => {
                    end_span = token.span;
                }
            }
        }
        let start_idx = open.span.start as usize;
        let end_idx = end_span.end as usize;
        let raw = self.source[start_idx..end_idx].to_string();
        Ok(Block {
            raw,
            statements: Vec::new(),
            span: Span::new(open.span.start, end_span.end),
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

    /// Accept either an identifier or a keyword token as an identifier
    /// This is needed in contexts like symbol maps where keywords can be used as names
    fn expect_ident_or_keyword(&mut self, expected: &'static str) -> Result<Token, ParseError> {
        let token = self.peek();
        match token.kind {
            TokenKind::Ident
            | TokenKind::KwModule
            | TokenKind::KwUse
            | TokenKind::KwType
            | TokenKind::KwFn
            | TokenKind::KwReturn
            | TokenKind::KwLet
            | TokenKind::KwIf
            | TokenKind::KwElse
            | TokenKind::KwWhile
            | TokenKind::KwCaps
            | TokenKind::KwAs
            | TokenKind::KwOnly
            | TokenKind::KwCtx
            | TokenKind::KwEff
            | TokenKind::KwMut
            | TokenKind::KwTrue
            | TokenKind::KwFalse => Ok(self.advance()),
            _ => Err(ParseError::Unexpected {
                expected,
                found: token.kind,
                span: token.span,
            }),
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
    use z1_ast::{FnDecl, Item, TypeExpr};

    #[test]
    fn parses_sample_cell() {
        let source = include_str!("../../../fixtures/cells/http_server.z1c");
        let module = parse_module(source).expect("module parses");
        let segments = module.path.as_str_vec().to_vec();
        assert_eq!(segments, vec!["http".to_string(), "server".to_string()]);
        assert_eq!(module.version.as_deref(), Some("1.0"));
        assert_eq!(module.ctx_budget, Some(128));
        assert_eq!(module.caps, vec!["net"]);
        assert_eq!(module.items.len(), 5);

        match &module.items[0] {
            Item::Symbol(sym) => {
                assert_eq!(sym.pairs.len(), 2);
            }
            other => panic!("expected symbol map, got {other:?}"),
        }

        match &module.items[1] {
            Item::Import(import) => {
                assert_eq!(import.path, "std/http");
                assert_eq!(import.alias.as_deref(), Some("H"));
                assert_eq!(import.only, vec!["listen", "Req", "Res"]);
            }
            other => panic!("expected import, got {other:?}"),
        }

        match &module.items[2] {
            Item::Type(ty) => {
                assert_eq!(ty.name, "Health");
                match &ty.expr {
                    TypeExpr::Record(fields) => {
                        assert_eq!(fields.len(), 2);
                    }
                    _ => panic!("expected record type"),
                }
            }
            other => panic!("expected type decl, got {other:?}"),
        }

        match &module.items[3] {
            Item::Fn(FnDecl { name, params, .. }) => {
                // Parser normalizes to long form: "h" -> "handler"
                assert_eq!(name, "handler");
                assert_eq!(params.len(), 1);
            }
            other => panic!("expected fn decl, got {other:?}"),
        }
    }
}
