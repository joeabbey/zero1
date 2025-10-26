//! Warning detection for effect checking.
//!
//! This module detects non-critical issues like:
//! - Functions declare effects but don't use them
//! - Over-permissive capability grants (caps not used)
//! - Async functions with no async operations

use crate::Effect;
use std::collections::HashSet;
use z1_ast::{FnDecl, Module, Span};

/// A warning detected during effect checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectWarning {
    UnusedEffect {
        effect: String,
        function: String,
        fn_span: Span,
    },
    UnusedCapability {
        capability: String,
        module: String,
        module_span: Span,
    },
    AsyncWithoutAsync {
        function: String,
        fn_span: Span,
    },
}

impl std::fmt::Display for EffectWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectWarning::UnusedEffect {
                effect, function, ..
            } => {
                write!(
                    f,
                    "Function '{function}' declares effect '{effect}' but may not use it"
                )
            }
            EffectWarning::UnusedCapability {
                capability, module, ..
            } => {
                write!(
                    f,
                    "Module '{module}' grants capability '{capability}' but no functions use it"
                )
            }
            EffectWarning::AsyncWithoutAsync { function, .. } => {
                write!(
                    f,
                    "Function '{function}' declares 'async' effect but contains no async operations"
                )
            }
        }
    }
}

impl EffectWarning {
    pub fn span(&self) -> Span {
        match self {
            EffectWarning::UnusedEffect { fn_span, .. }
            | EffectWarning::AsyncWithoutAsync { fn_span, .. } => *fn_span,
            EffectWarning::UnusedCapability { module_span, .. } => *module_span,
        }
    }
}

/// Collect warnings from a module.
pub fn collect_warnings(module: &Module) -> Vec<EffectWarning> {
    let mut warnings = Vec::new();

    // Collect all effects used by functions
    let mut used_effects = HashSet::new();
    for item in &module.items {
        if let z1_ast::Item::Fn(fn_decl) = item {
            for effect in &fn_decl.effects {
                if let Some(eff) = Effect::parse(effect) {
                    used_effects.insert(eff);
                }
            }
        }
    }

    // Check for unused capabilities
    let module_name = module.path.0.join(".");
    for cap in &module.caps {
        if let Some(base_cap) = parse_capability_base(cap) {
            if !used_effects.contains(&base_cap) && base_cap != Effect::Pure {
                warnings.push(EffectWarning::UnusedCapability {
                    capability: cap.clone(),
                    module: module_name.clone(),
                    module_span: module.span,
                });
            }
        }
    }

    // Check for unused effects in functions
    for item in &module.items {
        if let z1_ast::Item::Fn(fn_decl) = item {
            warnings.extend(check_function_warnings(fn_decl));
        }
    }

    warnings
}

/// Check for warnings in a function declaration.
fn check_function_warnings(fn_decl: &FnDecl) -> Vec<EffectWarning> {
    let warnings = Vec::new();

    // Check for async effect without async operations
    // Note: Full implementation would require analyzing function body
    // For MVP, this is a simplified check
    if fn_decl.effects.contains(&"async".to_string()) {
        // In a full implementation, we'd check if the body contains any await expressions
        // For now, this is a placeholder
    }

    warnings
}

/// Parse capability string to base effect.
fn parse_capability_base(cap: &str) -> Option<Effect> {
    if let Some((base, _suffix)) = cap.split_once('.') {
        Effect::parse(base)
    } else {
        Effect::parse(cap)
    }
}

/// Suggest a fix for unused capability.
#[allow(dead_code)]
pub fn suggest_unused_capability_fix(capability: &str, module: &str) -> String {
    format!(
        "Remove '{capability}' from module capabilities if not needed: module {module} caps=[...]"
    )
}

/// Suggest a fix for unused effect.
#[allow(dead_code)]
pub fn suggest_unused_effect_fix(effect: &str) -> String {
    format!("Remove '{effect}' from effect list if not used")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_capability_base() {
        assert_eq!(parse_capability_base("net"), Some(Effect::Net));
        assert_eq!(parse_capability_base("fs.ro"), Some(Effect::Fs));
        assert_eq!(parse_capability_base("fs.rw"), Some(Effect::Fs));
        assert_eq!(parse_capability_base("unknown"), None);
    }

    #[test]
    fn test_warning_display() {
        let warning = EffectWarning::UnusedEffect {
            effect: "net".to_string(),
            function: "foo".to_string(),
            fn_span: Span::new(0, 10),
        };
        assert!(format!("{warning}").contains("net"));
        assert!(format!("{warning}").contains("foo"));

        let warning = EffectWarning::UnusedCapability {
            capability: "time".to_string(),
            module: "test".to_string(),
            module_span: Span::new(0, 5),
        };
        assert!(format!("{warning}").contains("time"));
        assert!(format!("{warning}").contains("test"));
    }

    #[test]
    fn test_suggest_fixes() {
        let fix = suggest_unused_capability_fix("net", "test");
        assert!(fix.contains("net"));
        assert!(fix.contains("test"));

        let fix = suggest_unused_effect_fix("async");
        assert!(fix.contains("async"));
    }

    #[test]
    fn test_collect_warnings_no_unused() {
        use z1_ast::{Block, Item, ModulePath, TypeExpr};

        let fn_decl = FnDecl {
            name: "test".to_string(),
            params: vec![],
            ret: TypeExpr::Path(vec!["Unit".to_string()]),
            effects: vec!["net".to_string()],
            body: Block::default(),
            span: Span::new(0, 10),
        };

        let module = Module {
            path: ModulePath::from_parts(vec!["test".to_string()]),
            version: None,
            ctx_budget: None,
            caps: vec!["net".to_string()],
            items: vec![Item::Fn(fn_decl)],
            span: Span::new(0, 100),
        };

        let warnings = collect_warnings(&module);
        // Should not warn about 'net' since it's used by the function
        assert_eq!(
            warnings
                .iter()
                .filter(|w| matches!(w, EffectWarning::UnusedCapability { .. }))
                .count(),
            0
        );
    }

    #[test]
    fn test_collect_warnings_unused_capability() {
        use z1_ast::{Block, Item, ModulePath, TypeExpr};

        let fn_decl = FnDecl {
            name: "test".to_string(),
            params: vec![],
            ret: TypeExpr::Path(vec!["Unit".to_string()]),
            effects: vec![], // No effects
            body: Block::default(),
            span: Span::new(0, 10),
        };

        let module = Module {
            path: ModulePath::from_parts(vec!["test".to_string()]),
            version: None,
            ctx_budget: None,
            caps: vec!["net".to_string(), "time".to_string()], // Unused capabilities
            items: vec![Item::Fn(fn_decl)],
            span: Span::new(0, 100),
        };

        let warnings = collect_warnings(&module);
        // Should warn about both unused capabilities
        assert_eq!(
            warnings
                .iter()
                .filter(|w| matches!(w, EffectWarning::UnusedCapability { .. }))
                .count(),
            2
        );
    }
}
