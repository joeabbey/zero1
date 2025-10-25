//! Effect and capability checking for Zero1.
//!
//! This module enforces the effects system and capability budgets:
//! - Functions declare effects via `eff [...]` annotations
//! - Modules declare capabilities via `caps=[...]` in the header
//! - A function's effects must be a subset of the module's capabilities
//! - Pure functions (no effects or `eff [pure]`) can be called from anywhere

use std::collections::HashSet;
use thiserror::Error;
use z1_ast::{FnDecl, Module, Span};

#[derive(Debug, Error)]
pub enum EffectError {
    #[error("Function '{fn_name}' has effect '{effect}' but module '{module}' lacks capability '{effect}'")]
    MissingCapability {
        fn_name: String,
        module: String,
        effect: String,
        fn_span: Span,
        module_span: Span,
    },

    #[error("Function '{fn_name}' declares unknown effect '{effect}'")]
    UnknownEffect {
        fn_name: String,
        effect: String,
        fn_span: Span,
    },
}

/// Known effect types in Zero1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effect {
    Pure,
    Net,
    Fs,
    Time,
    Crypto,
    Env,
    Async,
    Unsafe,
}

impl Effect {
    /// Parse an effect identifier string into an Effect enum.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pure" => Some(Effect::Pure),
            "net" => Some(Effect::Net),
            "fs" => Some(Effect::Fs),
            "time" => Some(Effect::Time),
            "crypto" => Some(Effect::Crypto),
            "env" => Some(Effect::Env),
            "async" => Some(Effect::Async),
            "unsafe" => Some(Effect::Unsafe),
            _ => None,
        }
    }

    /// Convert effect to canonical string form.
    pub fn as_str(&self) -> &'static str {
        match self {
            Effect::Pure => "pure",
            Effect::Net => "net",
            Effect::Fs => "fs",
            Effect::Time => "time",
            Effect::Crypto => "crypto",
            Effect::Env => "env",
            Effect::Async => "async",
            Effect::Unsafe => "unsafe",
        }
    }
}

/// Parse a capability string into an Effect.
/// Capabilities use the same namespace as effects but may have fine-grained variants
/// like "fs.ro" and "fs.rw". For now, we normalize to the base effect.
fn parse_capability(cap: &str) -> Option<Effect> {
    // Handle fine-grained capabilities like fs.ro, fs.rw
    if let Some((base, _suffix)) = cap.split_once('.') {
        Effect::parse(base)
    } else {
        Effect::parse(cap)
    }
}

/// Result type for effect checking operations.
pub type Result<T> = std::result::Result<T, EffectError>;

/// Check all functions in a module against the module's declared capabilities.
///
/// # Rules
/// - Pure functions (no effects or `eff [pure]`) are always allowed
/// - For each non-pure function, all declared effects must be present in module caps
/// - Effects are matched case-insensitively
///
/// # Returns
/// - `Ok(())` if all functions have valid effect/capability combinations
/// - `Err(EffectError)` with the first violation found
pub fn check_module(module: &Module) -> Result<()> {
    // Parse module capabilities into a set
    let module_caps: HashSet<Effect> = module
        .caps
        .iter()
        .filter_map(|cap| parse_capability(cap))
        .collect();

    let module_name = module.path.0.join(".");

    // Check each function
    for item in &module.items {
        if let z1_ast::Item::Fn(fn_decl) = item {
            check_function(fn_decl, &module_caps, &module_name, module.span)?;
        }
    }

    Ok(())
}

/// Check a single function's effects against module capabilities.
fn check_function(
    fn_decl: &FnDecl,
    module_caps: &HashSet<Effect>,
    module_name: &str,
    module_span: Span,
) -> Result<()> {
    // If function has no effects, it's implicitly pure and always allowed
    if fn_decl.effects.is_empty() {
        return Ok(());
    }

    // Parse function effects
    let mut fn_effects = Vec::new();
    for eff_str in &fn_decl.effects {
        match Effect::parse(eff_str) {
            Some(eff) => fn_effects.push(eff),
            None => {
                return Err(EffectError::UnknownEffect {
                    fn_name: fn_decl.name.clone(),
                    effect: eff_str.clone(),
                    fn_span: fn_decl.span,
                });
            }
        }
    }

    // If function is pure, no capability check needed
    if fn_effects.len() == 1 && fn_effects[0] == Effect::Pure {
        return Ok(());
    }

    // Check each effect is present in module capabilities
    for effect in fn_effects {
        // Pure is always allowed
        if effect == Effect::Pure {
            continue;
        }

        if !module_caps.contains(&effect) {
            return Err(EffectError::MissingCapability {
                fn_name: fn_decl.name.clone(),
                module: module_name.to_string(),
                effect: effect.as_str().to_string(),
                fn_span: fn_decl.span,
                module_span,
            });
        }
    }

    Ok(())
}

/// Validate that effect A is a subset of effect B (for call-site checking).
///
/// This is used to verify that when function A calls function B,
/// A's effect set is a superset of B's effect set.
///
/// # Arguments
/// * `caller_effects` - The effect set of the calling function
/// * `callee_effects` - The effect set of the called function
///
/// # Returns
/// `true` if the call is valid (callee effects ⊆ caller effects)
pub fn can_call(caller_effects: &[Effect], callee_effects: &[Effect]) -> bool {
    let caller_set: HashSet<Effect> = caller_effects.iter().copied().collect();

    // Pure functions can always be called
    if callee_effects.is_empty() || (callee_effects.len() == 1 && callee_effects[0] == Effect::Pure)
    {
        return true;
    }

    // Check if all callee effects are present in caller
    callee_effects
        .iter()
        .filter(|&&e| e != Effect::Pure)
        .all(|e| caller_set.contains(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use z1_ast::{Block, Item, ModulePath, TypeExpr};

    fn make_module(caps: Vec<&str>, functions: Vec<FnDecl>) -> Module {
        Module {
            path: ModulePath::from_parts(vec!["test".to_string(), "module".to_string()]),
            version: Some("1.0".to_string()),
            ctx_budget: Some(128),
            caps: caps.into_iter().map(String::from).collect(),
            items: functions.into_iter().map(Item::Fn).collect(),
            span: Span::new(0, 100),
        }
    }

    fn make_fn(name: &str, effects: Vec<&str>) -> FnDecl {
        FnDecl {
            name: name.to_string(),
            params: vec![],
            ret: TypeExpr::Path(vec!["Unit".to_string()]),
            effects: effects.into_iter().map(String::from).collect(),
            body: Block::default(),
            span: Span::new(0, 10),
        }
    }

    #[test]
    fn test_pure_function_no_caps_needed() {
        let module = make_module(vec![], vec![make_fn("pure_fn", vec!["pure"])]);
        assert!(check_module(&module).is_ok());
    }

    #[test]
    fn test_no_effects_is_pure() {
        let module = make_module(vec![], vec![make_fn("implicit_pure", vec![])]);
        assert!(check_module(&module).is_ok());
    }

    #[test]
    fn test_net_effect_requires_net_cap() {
        let module = make_module(vec!["net"], vec![make_fn("network_fn", vec!["net"])]);
        assert!(check_module(&module).is_ok());
    }

    #[test]
    fn test_missing_capability_fails() {
        let module = make_module(vec![], vec![make_fn("network_fn", vec!["net"])]);
        let result = check_module(&module);
        assert!(result.is_err());

        if let Err(EffectError::MissingCapability { effect, .. }) = result {
            assert_eq!(effect, "net");
        } else {
            panic!("Expected MissingCapability error");
        }
    }

    #[test]
    fn test_multiple_effects_all_caps_present() {
        let module = make_module(
            vec!["net", "time", "fs"],
            vec![make_fn("complex_fn", vec!["net", "time"])],
        );
        assert!(check_module(&module).is_ok());
    }

    #[test]
    fn test_multiple_effects_missing_one_cap() {
        let module = make_module(
            vec!["net"],
            vec![make_fn("complex_fn", vec!["net", "time"])],
        );
        let result = check_module(&module);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_effect_fails() {
        let module = make_module(vec![], vec![make_fn("bad_fn", vec!["unknown_effect"])]);
        let result = check_module(&module);
        assert!(result.is_err());

        if let Err(EffectError::UnknownEffect { effect, .. }) = result {
            assert_eq!(effect, "unknown_effect");
        } else {
            panic!("Expected UnknownEffect error");
        }
    }

    #[test]
    fn test_case_insensitive_effects() {
        let module = make_module(vec!["net"], vec![make_fn("fn1", vec!["Net"])]);
        assert!(check_module(&module).is_ok());

        let module2 = make_module(vec!["Net"], vec![make_fn("fn2", vec!["net"])]);
        assert!(check_module(&module2).is_ok());
    }

    #[test]
    fn test_fine_grained_fs_capability() {
        // fs.ro should grant 'fs' effect
        let module = make_module(vec!["fs.ro"], vec![make_fn("read_fn", vec!["fs"])]);
        assert!(check_module(&module).is_ok());

        // fs.rw should also grant 'fs' effect
        let module2 = make_module(vec!["fs.rw"], vec![make_fn("write_fn", vec!["fs"])]);
        assert!(check_module(&module2).is_ok());
    }

    #[test]
    fn test_can_call_pure_from_anywhere() {
        assert!(can_call(&[], &[]));
        assert!(can_call(&[], &[Effect::Pure]));
        assert!(can_call(&[Effect::Net], &[Effect::Pure]));
    }

    #[test]
    fn test_can_call_subset_rule() {
        // Can call net function from net context
        assert!(can_call(&[Effect::Net], &[Effect::Net]));

        // Can call net function from net+time context
        assert!(can_call(&[Effect::Net, Effect::Time], &[Effect::Net]));

        // Cannot call net function from time-only context
        assert!(!can_call(&[Effect::Time], &[Effect::Net]));

        // Cannot call net function from pure context
        assert!(!can_call(&[], &[Effect::Net]));
    }

    #[test]
    fn test_can_call_multiple_effects() {
        // Can call net+time from net+time+fs context
        assert!(can_call(
            &[Effect::Net, Effect::Time, Effect::Fs],
            &[Effect::Net, Effect::Time]
        ));

        // Cannot call net+fs from net-only context
        assert!(!can_call(&[Effect::Net], &[Effect::Net, Effect::Fs]));
    }
}
