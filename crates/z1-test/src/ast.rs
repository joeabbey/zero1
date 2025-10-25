use serde::{Deserialize, Serialize};
use z1_ast::{Block, Ident, Span, TypeExpr};

/// Root structure for a `.z1t` test file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestFile {
    pub config: TestConfig,
    pub fixtures: Vec<Fixture>,
    pub specs: Vec<Spec>,
    pub props: Vec<Prop>,
    pub span: Span,
}

impl TestFile {
    pub fn new() -> Self {
        Self {
            config: TestConfig::default(),
            fixtures: Vec::new(),
            specs: Vec::new(),
            props: Vec::new(),
            span: Span::default(),
        }
    }
}

impl Default for TestFile {
    fn default() -> Self {
        Self::new()
    }
}

/// Test configuration (file-level)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TestConfig {
    pub timeout_ms: Option<u32>,
    pub tags_include: Vec<String>,
    pub tags_exclude: Vec<String>,
    pub parallel: Option<u32>,
    pub seed: Option<u64>,
}

/// Spec test (unit test with assertions)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec {
    pub name: String,
    pub attrs: TestAttrs,
    pub body: Block,
    pub span: Span,
}

/// Property-based test
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prop {
    pub name: String,
    pub attrs: TestAttrs,
    pub bindings: Vec<GenBinding>,
    pub runs: u32,
    pub seed: u64,
    pub body: Block,
    pub span: Span,
}

/// Property test generator binding
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenBinding {
    pub name: Ident,
    pub ty: TypeExpr,
    pub span: Span,
}

/// Test attributes (timeout, tags, skip, only)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TestAttrs {
    pub timeout_ms: Option<u32>,
    pub tags: Vec<String>,
    pub skip: bool,
    pub only: bool,
}

/// Fixture declaration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fixture {
    pub name: Ident,
    pub ty: Option<TypeExpr>,
    pub body: Block,
    pub span: Span,
}

/// Test assertion (simplified for MVP)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Assertion {
    Assert(String),           // assert <expr>
    AssertEq(String, String), // assert_eq(<expr>, <expr>)
    AssertNe(String, String), // assert_ne(<expr>, <expr>)
}
