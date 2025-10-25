use logos::Logos;
use z1_ast::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    KwModule,
    KwUse,
    KwAs,
    KwOnly,
    KwCtx,
    KwCaps,
    KwType,
    KwFn,
    KwEff,
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
    Sym,
    Hash,
    Unknown,
    Eof,
}

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
enum RawToken {
    #[regex(r"[ \t\r\n]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/", logos::skip)]
    Error,

    #[token("module")]
    #[token("m")]
    KwModule,

    #[token("use")]
    #[token("u")]
    KwUse,

    #[token("as")]
    KwAs,

    #[token("only")]
    KwOnly,

    #[token("ctx")]
    KwCtx,

    #[token("caps")]
    KwCaps,

    #[token("type")]
    #[token("t")]
    KwType,

    #[token("fn")]
    #[token("f")]
    KwFn,

    #[token("eff")]
    KwEff,

    #[regex(r"[A-Za-z_][A-Za-z0-9_\.]*")]
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
    #[token("#sym")]
    Sym,
    #[token("#")]
    Hash,
}

impl From<RawToken> for TokenKind {
    fn from(value: RawToken) -> Self {
        match value {
            RawToken::KwModule => TokenKind::KwModule,
            RawToken::KwUse => TokenKind::KwUse,
            RawToken::KwAs => TokenKind::KwAs,
            RawToken::KwOnly => TokenKind::KwOnly,
            RawToken::KwCtx => TokenKind::KwCtx,
            RawToken::KwCaps => TokenKind::KwCaps,
            RawToken::KwType => TokenKind::KwType,
            RawToken::KwFn => TokenKind::KwFn,
            RawToken::KwEff => TokenKind::KwEff,
            RawToken::Ident => TokenKind::Ident,
            RawToken::Number => TokenKind::Number,
            RawToken::String => TokenKind::String,
            RawToken::LParen => TokenKind::LParen,
            RawToken::RParen => TokenKind::RParen,
            RawToken::LBrace => TokenKind::LBrace,
            RawToken::RBrace => TokenKind::RBrace,
            RawToken::LBracket => TokenKind::LBracket,
            RawToken::RBracket => TokenKind::RBracket,
            RawToken::Comma => TokenKind::Comma,
            RawToken::Dot => TokenKind::Dot,
            RawToken::Colon => TokenKind::Colon,
            RawToken::Semi => TokenKind::Semi,
            RawToken::Eq => TokenKind::Eq,
            RawToken::Arrow => TokenKind::Arrow,
            RawToken::Sym => TokenKind::Sym,
            RawToken::Hash => TokenKind::Hash,
            RawToken::Error => TokenKind::Unknown,
        }
    }
}

/// Convert source text into a token stream (including a terminal EOF token).
pub fn lex(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut lexer = RawToken::lexer(source);
    while let Some(raw) = lexer.next() {
        let raw = raw.unwrap_or(RawToken::Error);
        let span = lexer.span();
        let token = Token {
            kind: TokenKind::from(raw),
            lexeme: lexer.slice().to_string(),
            span: Span::new(span.start as u32, span.end as u32),
        };
        tokens.push(token);
    }
    tokens.push(Token {
        kind: TokenKind::Eof,
        lexeme: String::new(),
        span: Span::new(source.len() as u32, source.len() as u32),
    });
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_module_header() {
        let input = "m http.server:1.0 ctx=128 caps=[net]";
        let tokens = lex(input);
        assert!(tokens.iter().any(|t| t.kind == TokenKind::KwModule));
        assert!(tokens.iter().any(|t| matches!(t.kind, TokenKind::Ident)));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::KwCtx));
        assert!(tokens.iter().any(|t| t.kind == TokenKind::KwCaps));
        assert_eq!(tokens.last().map(|t| t.kind), Some(TokenKind::Eof));
    }
}
