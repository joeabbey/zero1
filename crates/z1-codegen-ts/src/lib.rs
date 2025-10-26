//! TypeScript Code Generator for Zero1
//!
//! This crate generates TypeScript code from Zero1 IR. It provides a clean,
//! idiomatic TypeScript output that can be used in Node.js or browser environments.

use z1_ir::*;

/// TypeScript code generator
pub struct TsCodegen {
    output: String,
    indent_level: usize,
}

impl TsCodegen {
    /// Create a new TypeScript code generator
    pub fn new() -> Self {
        TsCodegen {
            output: String::new(),
            indent_level: 0,
        }
    }

    /// Generate TypeScript code from IR module
    pub fn generate(&mut self, module: &IrModule) -> String {
        self.output.clear();
        self.indent_level = 0;

        // File header comment
        self.write_line(&format!("// Generated from Z1 module: {}", module.name));
        self.write_line(&format!("// Version: {}", module.version));
        self.write_line("");

        // Imports
        for import in &module.imports {
            self.gen_import(import);
        }
        if !module.imports.is_empty() {
            self.write_line("");
        }

        // Type definitions
        for type_def in &module.types {
            self.gen_type_def(type_def);
            self.write_line("");
        }

        // Functions
        for func in &module.functions {
            self.gen_function(func);
            self.write_line("");
        }

        // Exports
        if !module.exports.is_empty() {
            self.write_line(&format!("export {{ {} }};", module.exports.join(", ")));
        }

        self.output.clone()
    }

    fn gen_import(&mut self, import: &IrImport) {
        let items = import.items.join(", ");
        let module_path = import.path.replace('/', "_");
        if !items.is_empty() {
            self.write_line(&format!("import {{ {items} }} from './{module_path}.js';"));
        } else {
            self.write_line(&format!("import './{module_path}.js';"));
        }
    }

    fn gen_type_def(&mut self, type_def: &IrTypeDef) {
        match &type_def.ty {
            IrType::Record(fields) => {
                self.write_line(&format!("export interface {} {{", type_def.name));
                self.indent_level += 1;
                for (field_name, field_type) in fields {
                    let field_ty = self.type_to_ts(field_type);
                    self.write_line(&format!("{field_name}: {field_ty};"));
                }
                self.indent_level -= 1;
                self.write_line("}");
            }
            IrType::Union(variants) => {
                let variant_types: Vec<String> = variants
                    .iter()
                    .map(|(name, ty)| {
                        if let Some(inner) = ty {
                            let inner_ts = self.type_to_ts(inner);
                            format!("{{ tag: '{name}', value: {inner_ts} }}")
                        } else {
                            format!("{{ tag: '{name}' }}")
                        }
                    })
                    .collect();
                self.write_line(&format!(
                    "export type {} = {};",
                    type_def.name,
                    variant_types.join(" | ")
                ));
            }
            _ => {
                let ty_ts = self.type_to_ts(&type_def.ty);
                self.write_line(&format!("export type {} = {ty_ts};", type_def.name));
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn type_to_ts(&self, ty: &IrType) -> String {
        match ty {
            IrType::Bool => "boolean".to_string(),
            IrType::Str => "string".to_string(),
            IrType::U16 | IrType::U32 | IrType::U64 => "number".to_string(),
            IrType::Unit => "void".to_string(),
            IrType::Named(name) => name.clone(),
            IrType::Record(fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(name, ty)| {
                        let ty_ts = self.type_to_ts(ty);
                        format!("{name}: {ty_ts}")
                    })
                    .collect();
                format!("{{ {} }}", field_strs.join(", "))
            }
            IrType::Union(variants) => {
                let variant_strs: Vec<String> = variants
                    .iter()
                    .map(|(name, ty)| {
                        if let Some(inner) = ty {
                            let inner_ts = self.type_to_ts(inner);
                            format!("{{ tag: '{name}', value: {inner_ts} }}")
                        } else {
                            format!("{{ tag: '{name}' }}")
                        }
                    })
                    .collect();
                variant_strs.join(" | ")
            }
            IrType::Generic { base, args } => {
                let arg_strs: Vec<String> = args.iter().map(|a| self.type_to_ts(a)).collect();
                let base_ts = self.type_to_ts(base);
                format!("{base_ts}<{}>", arg_strs.join(", "))
            }
        }
    }

    fn gen_function(&mut self, func: &IrFunction) {
        // Function signature
        let params: Vec<String> = func
            .params
            .iter()
            .map(|(name, ty)| {
                let ty_ts = self.type_to_ts(ty);
                format!("{name}: {ty_ts}")
            })
            .collect();
        let return_type = self.type_to_ts(&func.return_type);

        // Check for async effect
        let is_async = func
            .effects
            .iter()
            .any(|e| e.contains("async") || e.contains("Async"));
        let async_kw = if is_async { "async " } else { "" };

        self.write_line(&format!(
            "export {async_kw}function {}({}): {return_type} {{",
            func.name,
            params.join(", ")
        ));

        self.indent_level += 1;
        self.gen_block(&func.body);
        self.indent_level -= 1;

        self.write_line("}");
    }

    fn gen_block(&mut self, block: &IrBlock) {
        for stmt in &block.statements {
            self.gen_stmt(stmt);
        }
    }

    fn gen_stmt(&mut self, stmt: &IrStmt) {
        match stmt {
            IrStmt::Let {
                name,
                mutable,
                ty,
                value,
            } => {
                let var_kw = if *mutable { "let" } else { "const" };
                let type_annotation = ty
                    .as_ref()
                    .map(|t| {
                        let ty_ts = self.type_to_ts(t);
                        format!(": {ty_ts}")
                    })
                    .unwrap_or_default();
                let val_expr = self.gen_expr(value);
                self.write_line(&format!("{var_kw} {name}{type_annotation} = {val_expr};"));
            }
            IrStmt::Assign { target, value } => {
                let tgt = self.gen_expr(target);
                let val = self.gen_expr(value);
                self.write_line(&format!("{tgt} = {val};"));
            }
            IrStmt::If {
                cond,
                then_block,
                else_block,
            } => {
                let cond_expr = self.gen_expr(cond);
                self.write_line(&format!("if ({cond_expr}) {{"));
                self.indent_level += 1;
                self.gen_block(then_block);
                self.indent_level -= 1;
                if let Some(else_blk) = else_block {
                    self.write_line("} else {");
                    self.indent_level += 1;
                    self.gen_block(else_blk);
                    self.indent_level -= 1;
                }
                self.write_line("}");
            }
            IrStmt::While { cond, body } => {
                let cond_expr = self.gen_expr(cond);
                self.write_line(&format!("while ({cond_expr}) {{"));
                self.indent_level += 1;
                self.gen_block(body);
                self.indent_level -= 1;
                self.write_line("}");
            }
            IrStmt::Return { value } => {
                if let Some(val) = value {
                    let val_expr = self.gen_expr(val);
                    self.write_line(&format!("return {val_expr};"));
                } else {
                    self.write_line("return;");
                }
            }
            IrStmt::Expr(expr) => {
                let expr_str = self.gen_expr(expr);
                self.write_line(&format!("{expr_str};"));
            }
        }
    }

    fn gen_expr(&self, expr: &IrExpr) -> String {
        match expr {
            IrExpr::Var(name) => name.clone(),
            IrExpr::Literal(lit) => self.gen_literal(lit),
            IrExpr::BinOp { op, left, right } => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);
                let op_str = self.binop_to_ts(op);
                format!("{l} {op_str} {r}")
            }
            IrExpr::UnaryOp { op, expr } => {
                let op_str = self.unaryop_to_ts(op);
                let expr_str = self.gen_expr(expr);
                if *op == IrUnaryOp::Await {
                    format!("{op_str} {expr_str}")
                } else {
                    format!("{op_str}{expr_str}")
                }
            }
            IrExpr::Call { func, args } => {
                let arg_strs: Vec<String> = args.iter().map(|a| self.gen_expr(a)).collect();
                let func_str = self.gen_expr(func);
                format!("{func_str}({})", arg_strs.join(", "))
            }
            IrExpr::Field { base, field } => {
                let base_str = self.gen_expr(base);
                format!("{base_str}.{field}")
            }
            IrExpr::Record { fields } => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(name, val)| {
                        let val_str = self.gen_expr(val);
                        format!("{name}: {val_str}")
                    })
                    .collect();
                format!("{{ {} }}", field_strs.join(", "))
            }
            IrExpr::Path(segments) => segments.join("."),
        }
    }

    fn gen_literal(&self, lit: &IrLiteral) -> String {
        match lit {
            IrLiteral::Bool(b) => b.to_string(),
            IrLiteral::Str(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
            IrLiteral::U16(n) => n.to_string(),
            IrLiteral::U32(n) => n.to_string(),
            IrLiteral::U64(n) => n.to_string(),
            IrLiteral::Int(n) => n.to_string(),
            IrLiteral::Unit => "undefined".to_string(),
        }
    }

    fn binop_to_ts(&self, op: &IrBinOp) -> &str {
        match op {
            IrBinOp::Add => "+",
            IrBinOp::Sub => "-",
            IrBinOp::Mul => "*",
            IrBinOp::Div => "/",
            IrBinOp::Mod => "%",
            IrBinOp::Eq => "===",
            IrBinOp::Ne => "!==",
            IrBinOp::Lt => "<",
            IrBinOp::Le => "<=",
            IrBinOp::Gt => ">",
            IrBinOp::Ge => ">=",
            IrBinOp::And => "&&",
            IrBinOp::Or => "||",
        }
    }

    fn unaryop_to_ts(&self, op: &IrUnaryOp) -> &str {
        match op {
            IrUnaryOp::Neg => "-",
            IrUnaryOp::Not => "!",
            IrUnaryOp::Await => "await",
        }
    }

    fn write_line(&mut self, line: &str) {
        let indent = "  ".repeat(self.indent_level);
        self.output.push_str(&indent);
        self.output.push_str(line);
        self.output.push('\n');
    }
}

impl Default for TsCodegen {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate TypeScript code from IR module
pub fn generate_typescript(module: &IrModule) -> String {
    let mut codegen = TsCodegen::new();
    codegen.generate(module)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple_function() {
        let module = IrModule {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            imports: vec![],
            types: vec![],
            functions: vec![IrFunction {
                name: "greet".to_string(),
                params: vec![("name".to_string(), IrType::Str)],
                return_type: IrType::Str,
                effects: vec![],
                body: IrBlock {
                    statements: vec![IrStmt::Return {
                        value: Some(IrExpr::Literal(IrLiteral::Str("Hello".to_string()))),
                    }],
                },
            }],
            exports: vec!["greet".to_string()],
        };

        let ts = generate_typescript(&module);
        assert!(ts.contains("export function greet(name: string): string"));
        assert!(ts.contains("return \"Hello\";"));
        assert!(ts.contains("export { greet };"));
    }

    #[test]
    fn test_generate_type_interface() {
        let module = IrModule {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            imports: vec![],
            types: vec![IrTypeDef {
                name: "Point".to_string(),
                ty: IrType::Record(vec![
                    ("x".to_string(), IrType::U32),
                    ("y".to_string(), IrType::U32),
                ]),
            }],
            functions: vec![],
            exports: vec!["Point".to_string()],
        };

        let ts = generate_typescript(&module);
        assert!(ts.contains("export interface Point {"));
        assert!(ts.contains("x: number;"));
        assert!(ts.contains("y: number;"));
    }
}
