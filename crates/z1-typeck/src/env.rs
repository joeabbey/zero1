use crate::types::Type;
use std::collections::{HashMap, HashSet};
use z1_ast::Ident;

/// Typing context for variables, functions, and effects during type checking.
pub struct Context {
    /// Function signatures (name -> type)
    functions: HashMap<Ident, Type>,

    /// Variable types in current scope (name -> type)
    variables: HashMap<Ident, Type>,

    /// Effects available in the current context
    available_effects: HashSet<Ident>,

    /// Capabilities granted by the module
    granted_capabilities: HashSet<String>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            variables: HashMap::new(),
            available_effects: HashSet::new(),
            granted_capabilities: HashSet::new(),
        }
    }

    /// Register a function with its type signature.
    pub fn define_function(&mut self, name: Ident, ty: Type) {
        self.functions.insert(name, ty);
    }

    /// Lookup a function's type signature.
    pub fn lookup_function(&self, name: &Ident) -> Option<&Type> {
        self.functions.get(name)
    }

    /// Register a variable in the current scope.
    pub fn define_variable(&mut self, name: Ident, ty: Type) {
        self.variables.insert(name, ty);
    }

    /// Lookup a variable's type.
    pub fn lookup_variable(&self, name: &Ident) -> Option<&Type> {
        self.variables.get(name)
    }

    /// Add an effect to the available effects set.
    pub fn add_effect(&mut self, effect: Ident) {
        self.available_effects.insert(effect);
    }

    /// Check if an effect is available in the current context.
    pub fn has_effect(&self, effect: &Ident) -> bool {
        self.available_effects.contains(effect)
    }

    /// Get all available effects.
    pub fn available_effects(&self) -> &HashSet<Ident> {
        &self.available_effects
    }

    /// Set the capabilities granted by the module.
    pub fn set_capabilities(&mut self, caps: Vec<String>) {
        self.granted_capabilities = caps.into_iter().collect();
    }

    /// Check if a capability is granted.
    pub fn has_capability(&self, cap: &str) -> bool {
        self.granted_capabilities.contains(cap)
    }

    /// Create a new context inheriting functions and capabilities but with empty variables.
    /// This is useful when entering a new function scope.
    pub fn enter_function(&self, effects: &[Ident]) -> Self {
        let mut ctx = Self {
            functions: self.functions.clone(),
            variables: HashMap::new(),
            available_effects: effects.iter().cloned().collect(),
            granted_capabilities: self.granted_capabilities.clone(),
        };

        // Pure functions can always be called
        ctx.available_effects.insert("pure".to_string());

        ctx
    }

    /// Enter a nested block scope (inherits everything from parent).
    pub fn enter_block(&self) -> Self {
        Self {
            functions: self.functions.clone(),
            variables: self.variables.clone(),
            available_effects: self.available_effects.clone(),
            granted_capabilities: self.granted_capabilities.clone(),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// Map effects to required capabilities.
pub fn effect_to_capability(effect: &str) -> Option<&'static str> {
    match effect {
        "net" => Some("net"),
        "fs" => Some("fs"),
        "time" => Some("time"),
        "crypto" => Some("crypto"),
        "env" => Some("env"),
        "unsafe" => Some("unsafe"),
        "pure" => None,  // Pure effects require no capabilities
        "async" => None, // Async is a language feature, not a capability
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_definition_and_lookup() {
        let mut ctx = Context::new();
        let func_type = Type::Function {
            params: vec![Type::U32],
            ret: Box::new(Type::Bool),
            effects: vec!["pure".to_string()],
        };

        ctx.define_function("test_fn".to_string(), func_type.clone());
        assert_eq!(
            ctx.lookup_function(&"test_fn".to_string()),
            Some(&func_type)
        );
        assert_eq!(ctx.lookup_function(&"missing".to_string()), None);
    }

    #[test]
    fn test_effect_tracking() {
        let mut ctx = Context::new();
        assert!(!ctx.has_effect(&"net".to_string()));

        ctx.add_effect("net".to_string());
        assert!(ctx.has_effect(&"net".to_string()));
    }

    #[test]
    fn test_capability_tracking() {
        let mut ctx = Context::new();
        ctx.set_capabilities(vec!["net".to_string(), "time".to_string()]);

        assert!(ctx.has_capability("net"));
        assert!(ctx.has_capability("time"));
        assert!(!ctx.has_capability("fs"));
    }

    #[test]
    fn test_enter_function_scope() {
        let mut ctx = Context::new();
        ctx.define_function("outer".to_string(), Type::Bool);
        ctx.define_variable("x".to_string(), Type::U32);
        ctx.add_effect("net".to_string());

        let func_ctx = ctx.enter_function(&["fs".to_string()]);

        // Functions are inherited
        assert!(func_ctx.lookup_function(&"outer".to_string()).is_some());

        // Variables are NOT inherited (new scope)
        assert!(func_ctx.lookup_variable(&"x".to_string()).is_none());

        // Effects are replaced with new ones
        assert!(!func_ctx.has_effect(&"net".to_string()));
        assert!(func_ctx.has_effect(&"fs".to_string()));
        assert!(func_ctx.has_effect(&"pure".to_string())); // pure always available
    }

    #[test]
    fn test_effect_capability_mapping() {
        assert_eq!(effect_to_capability("net"), Some("net"));
        assert_eq!(effect_to_capability("fs"), Some("fs"));
        assert_eq!(effect_to_capability("pure"), None);
        assert_eq!(effect_to_capability("async"), None);
    }
}
