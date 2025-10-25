use logos::Logos;
use z1_ast::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestToken {
    pub kind: TestTokenKind,
    pub lexeme: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestTokenKind {
    // Test keywords
    KwSpec,
    KwProp,
    KwFixture,
    KwConfig,
    KwForAll,
    KwRuns,
    KwSeed,
    KwWith,
    KwTags,
    KwTimeout,
    KwSkip,
    KwOnly,
    KwAssert,
    KwAssertEq,
    KwAssertNe,

    // Standard tokens
    Ident,
    Number,
    String,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Dot,
    Colon,
    Semi,
    Eq,
    Arrow,

    Unknown,
    Eof,
}

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
enum RawTestToken {
    #[regex(r"[ \t\r\n]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/", logos::skip)]
    Error,

    #[token("spec")]
    KwSpec,

    #[token("prop")]
    KwProp,

    #[token("fixture")]
    KwFixture,

    #[token("config")]
    KwConfig,

    #[token("for_all")]
    KwForAll,

    #[token("runs")]
    KwRuns,

    #[token("seed")]
    KwSeed,

    #[token("with")]
    KwWith,

    #[token("tags")]
    KwTags,

    #[token("timeout")]
    #[token("timeout_ms")]
    KwTimeout,

    #[token("skip")]
    KwSkip,

    #[token("only")]
    KwOnly,

    #[token("assert")]
    KwAssert,

    #[token("assert_eq")]
    KwAssertEq,

    #[token("assert_ne")]
    KwAssertNe,

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident,

    #[regex(r"[0-9]+")]
    Number,

    #[regex(r#""([^"\\]|\\.)*""#)]
    String,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token(";")]
    Semi,
    #[token("=")]
    Eq,
    #[token("->")]
    Arrow,
}

impl From<RawTestToken> for TestTokenKind {
    fn from(value: RawTestToken) -> Self {
        match value {
            RawTestToken::KwSpec => TestTokenKind::KwSpec,
            RawTestToken::KwProp => TestTokenKind::KwProp,
            RawTestToken::KwFixture => TestTokenKind::KwFixture,
            RawTestToken::KwConfig => TestTokenKind::KwConfig,
            RawTestToken::KwForAll => TestTokenKind::KwForAll,
            RawTestToken::KwRuns => TestTokenKind::KwRuns,
            RawTestToken::KwSeed => TestTokenKind::KwSeed,
            RawTestToken::KwWith => TestTokenKind::KwWith,
            RawTestToken::KwTags => TestTokenKind::KwTags,
            RawTestToken::KwTimeout => TestTokenKind::KwTimeout,
            RawTestToken::KwSkip => TestTokenKind::KwSkip,
            RawTestToken::KwOnly => TestTokenKind::KwOnly,
            RawTestToken::KwAssert => TestTokenKind::KwAssert,
            RawTestToken::KwAssertEq => TestTokenKind::KwAssertEq,
            RawTestToken::KwAssertNe => TestTokenKind::KwAssertNe,
            RawTestToken::Ident => TestTokenKind::Ident,
            RawTestToken::Number => TestTokenKind::Number,
            RawTestToken::String => TestTokenKind::String,
            RawTestToken::LParen => TestTokenKind::LParen,
            RawTestToken::RParen => TestTokenKind::RParen,
            RawTestToken::LBrace => TestTokenKind::LBrace,
            RawTestToken::RBrace => TestTokenKind::RBrace,
            RawTestToken::LBracket => TestTokenKind::LBracket,
            RawTestToken::RBracket => TestTokenKind::RBracket,
            RawTestToken::Comma => TestTokenKind::Comma,
            RawTestToken::Dot => TestTokenKind::Dot,
            RawTestToken::Colon => TestTokenKind::Colon,
            RawTestToken::Semi => TestTokenKind::Semi,
            RawTestToken::Eq => TestTokenKind::Eq,
            RawTestToken::Arrow => TestTokenKind::Arrow,
            RawTestToken::Error => TestTokenKind::Unknown,
        }
    }
}

pub fn lex_test(source: &str) -> Vec<TestToken> {
    let mut tokens = Vec::new();
    let mut lexer = RawTestToken::lexer(source);

    while let Some(raw) = lexer.next() {
        let raw = raw.unwrap_or(RawTestToken::Error);
        let span = lexer.span();
        let token = TestToken {
            kind: TestTokenKind::from(raw),
            lexeme: lexer.slice().to_string(),
            span: Span::new(span.start as u32, span.end as u32),
        };
        tokens.push(token);
    }

    tokens.push(TestToken {
        kind: TestTokenKind::Eof,
        lexeme: String::new(),
        span: Span::new(source.len() as u32, source.len() as u32),
    });

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_spec_keyword() {
        let input = "spec \"test name\" { }";
        let tokens = lex_test(input);
        assert_eq!(tokens[0].kind, TestTokenKind::KwSpec);
        assert_eq!(tokens[1].kind, TestTokenKind::String);
        assert_eq!(tokens[2].kind, TestTokenKind::LBrace);
        assert_eq!(tokens[3].kind, TestTokenKind::RBrace);
    }

    #[test]
    fn lex_prop_keyword() {
        let input = "prop for_all (x: U32) runs 100 seed 42";
        let tokens = lex_test(input);
        assert!(tokens.iter().any(|t| t.kind == TestTokenKind::KwProp));
        assert!(tokens.iter().any(|t| t.kind == TestTokenKind::KwForAll));
        assert!(tokens.iter().any(|t| t.kind == TestTokenKind::KwRuns));
        assert!(tokens.iter().any(|t| t.kind == TestTokenKind::KwSeed));
    }

    #[test]
    fn lex_config_block() {
        let input = "config { timeout_ms: 3000 }";
        let tokens = lex_test(input);
        assert!(tokens.iter().any(|t| t.kind == TestTokenKind::KwConfig));
        assert!(tokens.iter().any(|t| t.kind == TestTokenKind::KwTimeout));
    }

    #[test]
    fn lex_assertions() {
        let input = "assert assert_eq assert_ne";
        let tokens = lex_test(input);
        assert_eq!(tokens[0].kind, TestTokenKind::KwAssert);
        assert_eq!(tokens[1].kind, TestTokenKind::KwAssertEq);
        assert_eq!(tokens[2].kind, TestTokenKind::KwAssertNe);
    }
}
