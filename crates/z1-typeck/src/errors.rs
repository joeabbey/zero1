use thiserror::Error;
use z1_ast::Span;

pub type TypeResult<T> = Result<T, TypeError>;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum TypeError {
    #[error("Type mismatch at {span:?}: expected {expected}, found {found}")]
    Mismatch {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("Undefined type '{name}' at {span:?}")]
    UndefinedType { name: String, span: Span },

    #[error("Undefined function '{name}' at {span:?}")]
    UndefinedFunction { name: String, span: Span },

    #[error("Undefined variable '{name}' at {span:?}")]
    UndefinedVariable { name: String, span: Span },

    #[error("Arity mismatch at {span:?}: expected {expected} parameters, found {found}")]
    ArityMismatch {
        expected: usize,
        found: usize,
        span: Span,
    },

    #[error("Record field mismatch: {message}")]
    RecordFieldMismatch { message: String },

    #[error(
        "Effect not permitted: function requires effect '{effect}' but context does not permit it"
    )]
    EffectNotPermitted { effect: String },

    #[error("Capability not granted: function requires capability '{cap}' but module does not declare it")]
    CapabilityNotGranted { cap: String },

    #[error("Invalid path: {message}")]
    InvalidPath { message: String },

    #[error("Duplicate definition: {message}")]
    DuplicateDefinition { message: String },
}

impl TypeError {
    pub fn mismatch(expected: String, found: String, span: Span) -> Self {
        Self::Mismatch {
            expected,
            found,
            span,
        }
    }

    pub fn undefined_type(name: String, span: Span) -> Self {
        Self::UndefinedType { name, span }
    }

    pub fn undefined_function(name: String, span: Span) -> Self {
        Self::UndefinedFunction { name, span }
    }

    pub fn arity_mismatch(expected: usize, found: usize, span: Span) -> Self {
        Self::ArityMismatch {
            expected,
            found,
            span,
        }
    }
}
