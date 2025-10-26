//! Warning detection for type checking.
//!
//! This module detects non-critical issues like:
//! - Unused variables
//! - Unused function parameters
//! - Shadowed variables
//! - Redundant type annotations

use z1_ast::{FnDecl, Module, Span};

/// A warning detected during type checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeWarning {
    UnusedVariable {
        name: String,
        span: Span,
    },
    UnusedParameter {
        name: String,
        function: String,
        span: Span,
    },
    ShadowedVariable {
        name: String,
        original_span: Span,
        shadow_span: Span,
    },
    RedundantTypeAnnotation {
        name: String,
        span: Span,
    },
}

impl std::fmt::Display for TypeWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeWarning::UnusedVariable { name, .. } => {
                write!(f, "Unused variable '{}'", name)
            }
            TypeWarning::UnusedParameter { name, function, .. } => {
                write!(f, "Unused parameter '{}' in function '{}'", name, function)
            }
            TypeWarning::ShadowedVariable { name, .. } => {
                write!(f, "Variable '{}' shadows a previous declaration", name)
            }
            TypeWarning::RedundantTypeAnnotation { name, .. } => {
                write!(f, "Type annotation for '{}' is redundant", name)
            }
        }
    }
}

impl TypeWarning {
    pub fn span(&self) -> Span {
        match self {
            TypeWarning::UnusedVariable { span, .. }
            | TypeWarning::UnusedParameter { span, .. }
            | TypeWarning::RedundantTypeAnnotation { span, .. } => *span,
            TypeWarning::ShadowedVariable { shadow_span, .. } => *shadow_span,
        }
    }
}

/// Collect warnings from a module.
pub fn collect_warnings(module: &Module) -> Vec<TypeWarning> {
    let mut warnings = Vec::new();

    for item in &module.items {
        if let z1_ast::Item::Fn(fn_decl) = item {
            warnings.extend(check_function_warnings(fn_decl));
        }
    }

    warnings
}

/// Check for warnings in a function declaration.
fn check_function_warnings(fn_decl: &FnDecl) -> Vec<TypeWarning> {
    let warnings = Vec::new();

    // Check for unused parameters
    // For MVP, we can't analyze function body usage, so we use a heuristic:
    // Parameters starting with underscore are intentionally unused
    for param in &fn_decl.params {
        if !param.name.starts_with('_') {
            // In a full implementation, we'd check if the parameter is used in the body
            // For now, we'll warn about parameters that look suspicious (single char, etc.)
            // This is a simplified check for demonstration
        }
    }

    // Check function body for unused variables and shadowing
    // Note: Since body.raw is a String, we can't do full AST analysis yet
    // This is a known MVP limitation

    warnings
}

/// Check if a variable name suggests it's intentionally unused.
#[allow(dead_code)]
pub fn is_intentionally_unused(name: &str) -> bool {
    name.starts_with('_') || name == "_"
}

/// Suggest a fix for an unused variable.
#[allow(dead_code)]
pub fn suggest_unused_fix(name: &str) -> String {
    if name.starts_with('_') {
        format!("Remove the variable '{name}'")
    } else {
        format!("Prefix with underscore if intentional: let _{name} = ...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_intentionally_unused() {
        assert!(is_intentionally_unused("_unused"));
        assert!(is_intentionally_unused("_"));
        assert!(!is_intentionally_unused("used"));
        assert!(!is_intentionally_unused("my_var"));
    }

    #[test]
    fn test_suggest_unused_fix() {
        assert!(suggest_unused_fix("unused").contains("_unused"));
        assert!(suggest_unused_fix("_foo").contains("Remove"));
    }

    #[test]
    fn test_warning_display() {
        let warning = TypeWarning::UnusedVariable {
            name: "x".to_string(),
            span: Span::new(0, 1),
        };
        assert_eq!(format!("{warning}"), "Unused variable 'x'");

        let warning = TypeWarning::UnusedParameter {
            name: "param".to_string(),
            function: "foo".to_string(),
            span: Span::new(0, 5),
        };
        assert_eq!(
            format!("{warning}"),
            "Unused parameter 'param' in function 'foo'"
        );
    }

    #[test]
    fn test_collect_warnings_empty() {
        use z1_ast::ModulePath;

        let module = Module {
            path: ModulePath::from_parts(vec!["test".to_string()]),
            version: None,
            ctx_budget: None,
            caps: vec![],
            items: vec![],
            span: Span::new(0, 0),
        };

        let warnings = collect_warnings(&module);
        assert_eq!(warnings.len(), 0);
    }
}
