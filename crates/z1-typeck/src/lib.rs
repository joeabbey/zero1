mod checker;
mod env;
mod errors;
mod types;
mod warnings;

pub use checker::TypeChecker;
pub use env::Context;
pub use errors::{TypeError, TypeResult};
pub use types::{Type, TypeEnv};
pub use warnings::{collect_warnings, TypeWarning};

use z1_ast::{Module, TypeExpr};

/// Type check a complete module and return any errors found.
pub fn check_module(module: &Module) -> TypeResult<()> {
    let mut checker = TypeChecker::new();
    checker.check_module(module)
}

/// Convert an AST TypeExpr to our internal Type representation.
/// This is used for testing and debugging.
pub fn type_from_ast(expr: &TypeExpr) -> Type {
    Type::from_ast(expr)
}
