use std::collections::{BTreeMap, HashMap};
use z1_ast::{Ident, TypeExpr as AstTypeExpr};

/// Internal representation of types for type checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    /// Primitive types
    Bool,
    Str,
    Unit,
    U16,
    U32,
    U64,

    /// Path-based type reference (may be aliased or fully qualified)
    Path(Vec<Ident>),

    /// Structural record type (field order independent for equality)
    Record(BTreeMap<Ident, Box<Type>>),

    /// Sum type / union (variant label to optional payload type)
    Sum(BTreeMap<Ident, Option<Box<Type>>>),

    /// Generic type application (e.g., `List<T>`)
    Generic {
        base: Box<Type>,
        args: Vec<Type>,
    },

    /// Function type (params, return type, effects)
    Function {
        params: Vec<Type>,
        ret: Box<Type>,
        effects: Vec<Ident>,
    },
}

impl Type {
    /// Convert an AST TypeExpr to our internal Type representation.
    pub fn from_ast(expr: &AstTypeExpr) -> Self {
        match expr {
            AstTypeExpr::Path(path) => {
                // Check if it's a primitive type
                if path.len() == 1 {
                    match path[0].as_str() {
                        "Bool" => return Type::Bool,
                        "Str" => return Type::Str,
                        "Unit" => return Type::Unit,
                        "U16" => return Type::U16,
                        "U32" => return Type::U32,
                        "U64" => return Type::U64,
                        _ => {}
                    }
                }
                Type::Path(path.clone())
            }
            AstTypeExpr::Record(fields) => {
                let mut map = BTreeMap::new();
                for field in fields {
                    let ty = Type::from_ast(&field.ty);
                    map.insert(field.name.clone(), Box::new(ty));
                }
                Type::Record(map)
            }
        }
    }

    /// Check if this type is a primitive type.
    pub fn is_primitive(&self) -> bool {
        matches!(
            self,
            Type::Bool | Type::Str | Type::Unit | Type::U16 | Type::U32 | Type::U64
        )
    }

    /// Get a display name for this type (for error messages).
    pub fn display_name(&self) -> String {
        match self {
            Type::Bool => "Bool".to_string(),
            Type::Str => "Str".to_string(),
            Type::Unit => "Unit".to_string(),
            Type::U16 => "U16".to_string(),
            Type::U32 => "U32".to_string(),
            Type::U64 => "U64".to_string(),
            Type::Path(path) => path.join("."),
            Type::Record(fields) => {
                let field_strs: Vec<_> = fields
                    .iter()
                    .map(|(name, ty)| format!("{}: {}", name, ty.display_name()))
                    .collect();
                format!("{{ {} }}", field_strs.join(", "))
            }
            Type::Sum(variants) => {
                let variant_strs: Vec<_> = variants
                    .iter()
                    .map(|(name, payload)| {
                        if let Some(ty) = payload {
                            format!("{}({})", name, ty.display_name())
                        } else {
                            name.clone()
                        }
                    })
                    .collect();
                variant_strs.join(" | ")
            }
            Type::Generic { base, args } => {
                let arg_strs: Vec<_> = args.iter().map(|t| t.display_name()).collect();
                format!("{}<{}>", base.display_name(), arg_strs.join(", "))
            }
            Type::Function {
                params,
                ret,
                effects,
            } => {
                let param_strs: Vec<_> = params.iter().map(|t| t.display_name()).collect();
                let eff_str = if effects.is_empty() {
                    String::new()
                } else {
                    format!(" eff [{}]", effects.join(", "))
                };
                format!(
                    "({}) -> {}{}",
                    param_strs.join(", "),
                    ret.display_name(),
                    eff_str
                )
            }
        }
    }

    /// Perform structural equality check for types.
    /// Records are equal if they have the same fields (order-independent).
    pub fn structural_eq(&self, other: &Type) -> bool {
        match (self, other) {
            // Primitives
            (Type::Bool, Type::Bool)
            | (Type::Str, Type::Str)
            | (Type::Unit, Type::Unit)
            | (Type::U16, Type::U16)
            | (Type::U32, Type::U32)
            | (Type::U64, Type::U64) => true,

            // Paths must match exactly
            (Type::Path(p1), Type::Path(p2)) => p1 == p2,

            // Records: structural equality (order-independent due to BTreeMap)
            (Type::Record(f1), Type::Record(f2)) => {
                if f1.len() != f2.len() {
                    return false;
                }
                for (name, ty1) in f1 {
                    match f2.get(name) {
                        Some(ty2) => {
                            if !ty1.structural_eq(ty2) {
                                return false;
                            }
                        }
                        None => return false,
                    }
                }
                true
            }

            // Sum types: must have same variants
            (Type::Sum(v1), Type::Sum(v2)) => {
                if v1.len() != v2.len() {
                    return false;
                }
                for (name, payload1) in v1 {
                    match v2.get(name) {
                        Some(payload2) => match (payload1, payload2) {
                            (Some(t1), Some(t2)) => {
                                if !t1.structural_eq(t2) {
                                    return false;
                                }
                            }
                            (None, None) => {}
                            _ => return false,
                        },
                        None => return false,
                    }
                }
                true
            }

            // Generics
            (Type::Generic { base: b1, args: a1 }, Type::Generic { base: b2, args: a2 }) => {
                if !b1.structural_eq(b2) {
                    return false;
                }
                if a1.len() != a2.len() {
                    return false;
                }
                a1.iter()
                    .zip(a2.iter())
                    .all(|(t1, t2)| t1.structural_eq(t2))
            }

            // Functions
            (
                Type::Function {
                    params: p1,
                    ret: r1,
                    effects: e1,
                },
                Type::Function {
                    params: p2,
                    ret: r2,
                    effects: e2,
                },
            ) => {
                if p1.len() != p2.len() {
                    return false;
                }
                if !r1.structural_eq(r2) {
                    return false;
                }
                // Effects must match (order matters for now)
                if e1 != e2 {
                    return false;
                }
                p1.iter()
                    .zip(p2.iter())
                    .all(|(t1, t2)| t1.structural_eq(t2))
            }

            _ => false,
        }
    }
}

/// Type environment for tracking type definitions and imported types.
pub struct TypeEnv {
    /// Type definitions in the current module
    types: HashMap<Ident, Type>,

    /// Imported types (qualified paths)
    imports: HashMap<Vec<Ident>, Type>,

    /// Import aliases (alias -> full path)
    aliases: HashMap<Ident, Vec<Ident>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            imports: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// Define a type in the current module.
    pub fn define_type(&mut self, name: Ident, ty: Type) {
        self.types.insert(name, ty);
    }

    /// Lookup a type by name (handles both local and imported types).
    pub fn lookup(&self, path: &[Ident]) -> Option<Type> {
        if path.len() == 1 {
            // Check local types first
            if let Some(ty) = self.types.get(&path[0]) {
                return Some(ty.clone());
            }
        }

        // Check if first component is an alias
        if let Some(full_path) = self.aliases.get(&path[0]) {
            let mut resolved_path = full_path.clone();
            resolved_path.extend_from_slice(&path[1..]);
            if let Some(ty) = self.imports.get(&resolved_path) {
                return Some(ty.clone());
            }
        }

        // Try as a full path
        self.imports.get(path).cloned()
    }

    /// Register an import alias.
    pub fn register_alias(&mut self, alias: Ident, full_path: Vec<Ident>) {
        self.aliases.insert(alias, full_path);
    }

    /// Register an imported type.
    pub fn register_import(&mut self, path: Vec<Ident>, ty: Type) {
        self.imports.insert(path, ty);
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_equality() {
        assert!(Type::Bool.structural_eq(&Type::Bool));
        assert!(Type::U32.structural_eq(&Type::U32));
        assert!(!Type::Bool.structural_eq(&Type::Str));
    }

    #[test]
    fn test_record_structural_equality() {
        let mut fields1 = BTreeMap::new();
        fields1.insert("x".to_string(), Box::new(Type::U32));
        fields1.insert("y".to_string(), Box::new(Type::Bool));

        let mut fields2 = BTreeMap::new();
        fields2.insert("y".to_string(), Box::new(Type::Bool));
        fields2.insert("x".to_string(), Box::new(Type::U32));

        let rec1 = Type::Record(fields1);
        let rec2 = Type::Record(fields2);

        assert!(rec1.structural_eq(&rec2));
    }

    #[test]
    fn test_record_field_mismatch() {
        let mut fields1 = BTreeMap::new();
        fields1.insert("x".to_string(), Box::new(Type::U32));

        let mut fields2 = BTreeMap::new();
        fields2.insert("y".to_string(), Box::new(Type::U32));

        let rec1 = Type::Record(fields1);
        let rec2 = Type::Record(fields2);

        assert!(!rec1.structural_eq(&rec2));
    }

    #[test]
    fn test_path_equality() {
        let path1 = Type::Path(vec!["http".to_string(), "Req".to_string()]);
        let path2 = Type::Path(vec!["http".to_string(), "Req".to_string()]);
        let path3 = Type::Path(vec!["http".to_string(), "Res".to_string()]);

        assert!(path1.structural_eq(&path2));
        assert!(!path1.structural_eq(&path3));
    }

    #[test]
    fn test_display_name() {
        let ty = Type::Bool;
        assert_eq!(ty.display_name(), "Bool");

        let mut fields = BTreeMap::new();
        fields.insert("ok".to_string(), Box::new(Type::Bool));
        fields.insert("msg".to_string(), Box::new(Type::Str));
        let rec = Type::Record(fields);

        // Display name includes fields
        let display = rec.display_name();
        assert!(display.contains("ok: Bool"));
        assert!(display.contains("msg: Str"));
    }
}
