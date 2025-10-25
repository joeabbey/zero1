use crate::env::{effect_to_capability, Context};
use crate::errors::{TypeError, TypeResult};
use crate::types::{Type, TypeEnv};
use std::collections::HashSet;
use z1_ast::{FnDecl, Import, Item, Module, TypeDecl};

pub struct TypeChecker {
    type_env: TypeEnv,
    context: Context,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            type_env: TypeEnv::new(),
            context: Context::new(),
        }
    }

    /// Type check a complete module.
    pub fn check_module(&mut self, module: &Module) -> TypeResult<()> {
        // Set capabilities from module header
        self.context.set_capabilities(module.caps.clone());

        // First pass: collect all type and function declarations
        for item in &module.items {
            match item {
                Item::Type(type_decl) => {
                    self.collect_type_decl(type_decl)?;
                }
                Item::Fn(fn_decl) => {
                    self.collect_function_signature(fn_decl)?;
                }
                Item::Import(import) => {
                    self.process_import(import)?;
                }
                Item::Symbol(_) => {
                    // Symbol maps are formatting-only, ignored for type checking
                }
            }
        }

        // Second pass: type check function bodies and verify types
        for item in &module.items {
            if let Item::Fn(fn_decl) = item {
                self.check_function(fn_decl)?;
            }
        }

        Ok(())
    }

    /// Collect a type declaration into the type environment.
    fn collect_type_decl(&mut self, decl: &TypeDecl) -> TypeResult<()> {
        let ty = Type::from_ast(&decl.expr);
        self.type_env.define_type(decl.name.clone(), ty);
        Ok(())
    }

    /// Collect a function signature into the context.
    fn collect_function_signature(&mut self, decl: &FnDecl) -> TypeResult<()> {
        // Check that required capabilities are granted
        for effect in &decl.effects {
            if let Some(cap) = effect_to_capability(effect) {
                if !self.context.has_capability(cap) {
                    return Err(TypeError::CapabilityNotGranted {
                        cap: cap.to_string(),
                    });
                }
            }
        }

        let param_types: Vec<Type> = decl
            .params
            .iter()
            .map(|p| self.resolve_type(&p.ty, decl.span))
            .collect::<TypeResult<Vec<_>>>()?;

        let ret_type = self.resolve_type(&decl.ret, decl.span)?;

        let func_type = Type::Function {
            params: param_types,
            ret: Box::new(ret_type),
            effects: decl.effects.clone(),
        };

        self.context.define_function(decl.name.clone(), func_type);
        Ok(())
    }

    /// Process an import statement (stubbed for MVP - full resolution requires module system).
    fn process_import(&mut self, import: &Import) -> TypeResult<()> {
        // For MVP, we register the alias if present
        if let Some(alias) = &import.alias {
            // We don't have the full module system yet, so we just track the alias
            // In a complete implementation, this would load the imported module and
            // resolve the types from it
            let import_path = import.path.split('/').map(|s| s.to_string()).collect();
            self.type_env.register_alias(alias.clone(), import_path);
        }

        // For now, we stub imported types as Path types
        // A full implementation would resolve these from the imported module
        for name in &import.only {
            let qualified_name = if let Some(alias) = &import.alias {
                vec![alias.clone(), name.clone()]
            } else {
                vec![name.clone()]
            };

            // Register as an opaque path type for now
            let imported_type = Type::Path(qualified_name.clone());
            self.type_env.register_import(qualified_name, imported_type);
        }

        Ok(())
    }

    /// Resolve a TypeExpr to a Type, handling path resolution.
    fn resolve_type(&self, expr: &z1_ast::TypeExpr, _span: z1_ast::Span) -> TypeResult<Type> {
        let ty = Type::from_ast(expr);

        // If it's a path type, try to resolve it
        if let Type::Path(ref path) = ty {
            // Check if it's already a primitive
            if ty.is_primitive() {
                return Ok(ty);
            }

            // Try to look up in type environment
            if let Some(resolved) = self.type_env.lookup(path) {
                return Ok(resolved);
            }

            // If not found, it might be an imported type that we're treating as opaque
            // For MVP, we allow path types to remain unresolved
            Ok(ty)
        } else {
            Ok(ty)
        }
    }

    /// Type check a function declaration.
    fn check_function(&mut self, decl: &FnDecl) -> TypeResult<()> {
        // Create a new context for this function scope
        let mut func_ctx = self.context.enter_function(&decl.effects);

        // Add parameters to the function context
        for param in &decl.params {
            let param_type = self.resolve_type(&param.ty, param.span)?;
            func_ctx.define_variable(param.name.clone(), param_type);
        }

        // For MVP, we don't have full statement AST yet (body.raw is String)
        // So we can't type check the function body in detail
        // This is a known limitation documented in PROGRESS.md

        // We do basic validation: check that the function signature is well-formed
        let ret_type = self.resolve_type(&decl.ret, decl.span)?;

        // Verify that the return type is valid
        if let Type::Path(ref path) = ret_type {
            if !ret_type.is_primitive() && self.type_env.lookup(path).is_none() {
                // Allow unresolved paths for now (might be imported types)
                // In a full implementation, this would be an error
            }
        }

        Ok(())
    }

    /// Check that a call site's effects are compatible with the calling context.
    pub fn check_effect_compatibility(
        &self,
        required_effects: &[String],
        available_effects: &HashSet<String>,
    ) -> TypeResult<()> {
        for effect in required_effects {
            if effect != "pure" && !available_effects.contains(effect) {
                return Err(TypeError::EffectNotPermitted {
                    effect: effect.clone(),
                });
            }
        }
        Ok(())
    }

    /// Check structural type equality (public for testing).
    pub fn check_type_equality(
        &self,
        expected: &Type,
        found: &Type,
        span: z1_ast::Span,
    ) -> TypeResult<()> {
        if !expected.structural_eq(found) {
            return Err(TypeError::mismatch(
                expected.display_name(),
                found.display_name(),
                span,
            ));
        }
        Ok(())
    }

    /// Check function call arity and types (public for testing).
    pub fn check_call(
        &self,
        func_type: &Type,
        args: &[Type],
        span: z1_ast::Span,
    ) -> TypeResult<Type> {
        match func_type {
            Type::Function {
                params,
                ret,
                effects,
            } => {
                // Check arity
                if params.len() != args.len() {
                    return Err(TypeError::arity_mismatch(params.len(), args.len(), span));
                }

                // Check parameter types
                for (param_ty, arg_ty) in params.iter().zip(args.iter()) {
                    if !param_ty.structural_eq(arg_ty) {
                        return Err(TypeError::mismatch(
                            param_ty.display_name(),
                            arg_ty.display_name(),
                            span,
                        ));
                    }
                }

                // Check effects
                self.check_effect_compatibility(effects, self.context.available_effects())?;

                Ok((**ret).clone())
            }
            _ => Err(TypeError::Mismatch {
                expected: "function type".to_string(),
                found: func_type.display_name(),
                span,
            }),
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use z1_ast::Span;

    fn make_span() -> Span {
        Span::new(0, 0)
    }

    #[test]
    fn test_primitive_type_equality() {
        let checker = TypeChecker::new();
        let span = make_span();

        assert!(checker
            .check_type_equality(&Type::Bool, &Type::Bool, span)
            .is_ok());
        assert!(checker
            .check_type_equality(&Type::U32, &Type::U32, span)
            .is_ok());
        assert!(checker
            .check_type_equality(&Type::Bool, &Type::Str, span)
            .is_err());
    }

    #[test]
    fn test_record_structural_equality() {
        let checker = TypeChecker::new();
        let span = make_span();

        let mut fields1 = BTreeMap::new();
        fields1.insert("x".to_string(), Box::new(Type::U32));
        fields1.insert("y".to_string(), Box::new(Type::Bool));

        let mut fields2 = BTreeMap::new();
        fields2.insert("y".to_string(), Box::new(Type::Bool));
        fields2.insert("x".to_string(), Box::new(Type::U32));

        let rec1 = Type::Record(fields1);
        let rec2 = Type::Record(fields2);

        assert!(checker.check_type_equality(&rec1, &rec2, span).is_ok());
    }

    #[test]
    fn test_function_call_arity_check() {
        let checker = TypeChecker::new();
        let span = make_span();

        let func_type = Type::Function {
            params: vec![Type::U32, Type::Bool],
            ret: Box::new(Type::Str),
            effects: vec!["pure".to_string()],
        };

        // Correct arity
        let args = vec![Type::U32, Type::Bool];
        assert!(checker.check_call(&func_type, &args, span).is_ok());

        // Wrong arity
        let args = vec![Type::U32];
        assert!(checker.check_call(&func_type, &args, span).is_err());
    }

    #[test]
    fn test_function_call_type_check() {
        let checker = TypeChecker::new();
        let span = make_span();

        let func_type = Type::Function {
            params: vec![Type::U32],
            ret: Box::new(Type::Bool),
            effects: vec!["pure".to_string()],
        };

        // Correct type
        let args = vec![Type::U32];
        let result = checker.check_call(&func_type, &args, span);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Type::Bool);

        // Wrong type
        let args = vec![Type::Str];
        assert!(checker.check_call(&func_type, &args, span).is_err());
    }

    #[test]
    fn test_effect_compatibility() {
        let checker = TypeChecker::new();
        let mut available = HashSet::new();
        available.insert("pure".to_string());
        available.insert("net".to_string());

        // Pure is always allowed
        assert!(checker
            .check_effect_compatibility(&["pure".to_string()], &available)
            .is_ok());

        // Net is available
        assert!(checker
            .check_effect_compatibility(&["net".to_string()], &available)
            .is_ok());

        // Fs is not available
        assert!(checker
            .check_effect_compatibility(&["fs".to_string()], &available)
            .is_err());
    }

    #[test]
    fn test_capability_checking() {
        let mut checker = TypeChecker::new();
        checker.context.set_capabilities(vec!["net".to_string()]);

        let fn_decl = FnDecl {
            name: "test_fn".to_string(),
            params: vec![],
            ret: z1_ast::TypeExpr::Path(vec!["Unit".to_string()]),
            effects: vec!["net".to_string()],
            body: z1_ast::Block::default(),
            span: make_span(),
        };

        // Should succeed - net capability is granted
        assert!(checker.collect_function_signature(&fn_decl).is_ok());

        // Should fail - fs capability is not granted
        let fn_decl_fs = FnDecl {
            name: "test_fn_fs".to_string(),
            params: vec![],
            ret: z1_ast::TypeExpr::Path(vec!["Unit".to_string()]),
            effects: vec!["fs".to_string()],
            body: z1_ast::Block::default(),
            span: make_span(),
        };

        assert!(checker.collect_function_signature(&fn_decl_fs).is_err());
    }
}
