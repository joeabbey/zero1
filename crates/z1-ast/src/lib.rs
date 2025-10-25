use serde::{Deserialize, Serialize};

pub type Ident = String;

/// Byte-offset span within a source string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

/// Fully-qualified module path, e.g., `http.server`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ModulePath(pub Vec<Ident>);

impl ModulePath {
    pub fn from_parts(parts: Vec<Ident>) -> Self {
        Self(parts)
    }

    pub fn push(&mut self, ident: impl Into<Ident>) {
        self.0.push(ident.into());
    }

    pub fn as_str_vec(&self) -> &[Ident] {
        &self.0
    }
}

/// Parsed module representation (header + top-level items).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Module {
    pub path: ModulePath,
    pub version: Option<String>,
    pub ctx_budget: Option<u32>,
    pub caps: Vec<String>,
    pub items: Vec<Item>,
    pub span: Span,
}

impl Module {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        path: ModulePath,
        version: Option<String>,
        ctx_budget: Option<u32>,
        caps: Vec<String>,
        items: Vec<Item>,
        span: Span,
    ) -> Self {
        Self {
            path,
            version,
            ctx_budget,
            caps,
            items,
            span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    Import(Import),
    Symbol(SymbolMap),
    Type(TypeDecl),
    Fn(FnDecl),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Import {
    pub path: String,
    pub alias: Option<Ident>,
    pub only: Vec<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SymbolMap {
    pub pairs: Vec<SymbolPair>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolPair {
    pub long: Ident,
    pub short: Ident,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeDecl {
    pub name: Ident,
    pub expr: TypeExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeExpr {
    Path(Vec<Ident>),
    Record(Vec<RecordField>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordField {
    pub name: Ident,
    pub ty: Box<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FnDecl {
    pub name: Ident,
    pub params: Vec<Param>,
    pub ret: TypeExpr,
    pub effects: Vec<Ident>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Param {
    pub name: Ident,
    pub ty: TypeExpr,
    pub span: Span,
}

/// Placeholder for statements until the full AST exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Block {
    pub raw: String,
    pub span: Span,
}
