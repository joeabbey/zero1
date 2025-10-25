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

/// Block of statements (function body or control flow body)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Block {
    /// Raw source text captured during parsing (temporary until full formatter/AST round-trip)
    pub raw: String,
    /// Parsed statements (may be empty until statement parsing is complete)
    pub statements: Vec<Stmt>,
    pub span: Span,
}

/// Statement types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stmt {
    Let(LetStmt),
    Assign(AssignStmt),
    If(IfStmt),
    While(WhileStmt),
    Return(ReturnStmt),
    Expr(ExprStmt),
}

/// Let binding: `let x: Type = expr;` or `let mut x = expr;`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LetStmt {
    pub mutable: bool,
    pub name: Ident,
    pub ty: Option<TypeExpr>,
    pub init: Expr,
    pub span: Span,
}

/// Assignment: `x = expr;` or `obj.field = expr;`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignStmt {
    pub target: Expr, // Should be an LValue (identifier or field access)
    pub value: Expr,
    pub span: Span,
}

/// If statement: `if cond { ... } else { ... }`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_block: Block,
    pub else_block: Option<Box<ElseBlock>>,
    pub span: Span,
}

/// Else block can be another block or another if statement (else-if)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElseBlock {
    Block(Block),
    If(IfStmt),
}

/// While loop: `while cond { ... }`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WhileStmt {
    pub cond: Expr,
    pub body: Block,
    pub span: Span,
}

/// Return statement: `return expr;` or `return;`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

/// Expression statement: `expr;`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

/// Expressions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expr {
    /// Identifier: `x`
    Ident(Ident, Span),
    /// Literal value: `42`, `"hello"`, `true`
    Literal(Literal, Span),
    /// Binary operation: `a + b`, `x == y`
    BinOp {
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>,
        span: Span,
    },
    /// Unary operation: `-x`, `!flag`, `await task`
    UnaryOp {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    /// Function call: `foo(a, b)`
    Call {
        func: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    /// Field access: `obj.field`
    Field {
        base: Box<Expr>,
        field: Ident,
        span: Span,
    },
    /// Record initialization: `Point { x: 1, y: 2 }`
    Record { fields: Vec<RecordInit>, span: Span },
    /// Qualified path: `H.Req`, `std.io.File`
    Path(Vec<Ident>, Span),
    /// Parenthesized expression: `(expr)`
    Paren(Box<Expr>, Span),
}

/// Record field initialization in an expression
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordInit {
    pub name: Ident,
    pub value: Expr,
    pub span: Span,
}

/// Literal values
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Literal {
    Bool(bool),
    Str(String),
    U16(u16),
    U32(u32),
    U64(u64),
    Int(i64), // Unsuffixed integer
    Unit,     // ()
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,   // -
    Not,   // !
    Await, // await
}
