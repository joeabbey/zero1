use std::collections::HashMap;

use thiserror::Error;
use z1_ast::{
    FnDecl, Import, Item, Module, Param, RecordField, SymbolMap, SymbolPair, TypeDecl, TypeExpr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Compact,
    Relaxed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymMapStyle {
    Respect,
    Reflow,
}

impl Default for SymMapStyle {
    fn default() -> Self {
        Self::Respect
    }
}

#[derive(Debug, Clone)]
pub struct FmtOptions {
    pub symmap_style: SymMapStyle,
}

impl Default for FmtOptions {
    fn default() -> Self {
        Self {
            symmap_style: SymMapStyle::Respect,
        }
    }
}

#[derive(Debug, Error)]
pub enum FmtError {
    #[error("formatter invariant violated: {0}")]
    Unsupported(&'static str),
}

pub fn format_module(
    module: &Module,
    mode: Mode,
    options: &FmtOptions,
) -> Result<String, FmtError> {
    let symbols = SymbolTable::new(module, options.symmap_style);
    let mut formatter = Formatter::new(module, mode, symbols);
    formatter.write_module_header();
    formatter.write_items()?;
    formatter.finish();
    Ok(formatter.buf)
}

struct Formatter<'a> {
    module: &'a Module,
    mode: Mode,
    buf: String,
    symbols: SymbolTable,
    sections_emitted: usize,
}

impl<'a> Formatter<'a> {
    fn new(module: &'a Module, mode: Mode, symbols: SymbolTable) -> Self {
        Self {
            module,
            mode,
            buf: String::with_capacity(256),
            symbols,
            sections_emitted: 0,
        }
    }

    fn finish(&mut self) {
        if !self.buf.ends_with('\n') {
            self.buf.push('\n');
        }
    }

    fn write_module_header(&mut self) {
        match self.mode {
            Mode::Compact => self.write_compact_header(),
            Mode::Relaxed => self.write_relaxed_header(),
        }
    }

    fn write_items(&mut self) -> Result<(), FmtError> {
        for item in &self.module.items {
            match item {
                Item::Import(import) => {
                    self.section_break();
                    self.write_import(import);
                }
                Item::Symbol(sym) => {
                    self.section_break();
                    self.write_symbol_map(sym);
                }
                Item::Type(ty) => {
                    self.section_break();
                    self.write_type_decl(ty);
                }
                Item::Fn(func) => {
                    self.section_break();
                    self.write_fn_decl(func);
                }
            }
        }
        Ok(())
    }

    fn write_compact_header(&mut self) {
        self.buf.push_str("m ");
        self.buf.push_str(&self.module_path());
        if let Some(version) = &self.module.version {
            self.buf.push(':');
            self.buf.push_str(version);
        }
        if let Some(ctx) = self.module.ctx_budget {
            self.buf.push(' ');
            self.buf.push_str("ctx=");
            self.buf.push_str(&ctx.to_string());
        }
        if !self.module.caps.is_empty() {
            self.buf.push(' ');
            self.buf.push_str("caps=");
            self.buf.push('[');
            self.buf.push_str(&self.module.caps.join(","));
            self.buf.push(']');
        }
        self.buf.push('\n');
    }

    fn write_relaxed_header(&mut self) {
        self.buf.push_str("module ");
        self.buf.push_str(&self.module_path());
        if let Some(version) = &self.module.version {
            self.buf.push_str(" : ");
            self.buf.push_str(version);
        }
        self.buf.push('\n');
        if let Some(ctx) = self.module.ctx_budget {
            self.buf.push_str("  ctx = ");
            self.buf.push_str(&ctx.to_string());
            self.buf.push('\n');
        }
        if !self.module.caps.is_empty() {
            self.buf.push_str("  caps = [");
            self.buf.push_str(&self.module.caps.join(", "));
            self.buf.push_str("]\n");
        }
    }

    fn write_import(&mut self, import: &Import) {
        match self.mode {
            Mode::Compact => {
                self.buf.push_str("u ");
            }
            Mode::Relaxed => {
                self.buf.push_str("use ");
            }
        }
        self.buf.push('"');
        self.buf.push_str(&import.path);
        self.buf.push('"');
        if let Some(alias) = &import.alias {
            let alias_display = self.symbols.display_ident(alias, self.mode);
            self.buf.push_str(" as ");
            self.buf.push_str(&alias_display);
        }
        if !import.only.is_empty() {
            self.buf.push_str(" only [");
            self.buf.push_str(&import.only.join(", "));
            self.buf.push(']');
        }
        self.buf.push('\n');
    }

    fn write_symbol_map(&mut self, map: &SymbolMap) {
        let pairs = self.symbols.ordered_pairs(&map.pairs);
        if pairs.is_empty() {
            return;
        }
        match self.mode {
            Mode::Compact => {
                self.buf.push_str("#sym { ");
                for (idx, (long, short)) in pairs.iter().enumerate() {
                    if idx > 0 {
                        self.buf.push_str(", ");
                    }
                    self.buf.push_str(long);
                    self.buf.push_str(": ");
                    self.buf.push_str(short);
                }
                self.buf.push_str(" }\n");
            }
            Mode::Relaxed => {
                self.buf.push_str("// SymbolMap: { ");
                for (idx, (long, short)) in pairs.iter().enumerate() {
                    if idx > 0 {
                        self.buf.push_str(", ");
                    }
                    self.buf.push_str(long);
                    self.buf.push_str(" ↔ ");
                    self.buf.push_str(short);
                }
                self.buf.push_str(" }\n");
                self.buf.push_str("#sym { ");
                for (idx, (long, short)) in pairs.iter().enumerate() {
                    if idx > 0 {
                        self.buf.push_str(", ");
                    }
                    self.buf.push_str(long);
                    self.buf.push_str(": ");
                    self.buf.push_str(short);
                }
                self.buf.push_str(" }\n");
            }
        }
    }

    fn write_type_decl(&mut self, decl: &TypeDecl) {
        match self.mode {
            Mode::Compact => self.buf.push_str("t "),
            Mode::Relaxed => self.buf.push_str("type "),
        }
        let name = self.symbols.display_ident(&decl.name, self.mode);
        self.buf.push_str(&name);
        self.buf.push_str(" = ");
        self.buf.push_str(&self.format_type_expr(&decl.expr));
        self.buf.push('\n');
    }

    fn write_fn_decl(&mut self, decl: &FnDecl) {
        let kw = match self.mode {
            Mode::Compact => "f",
            Mode::Relaxed => "fn",
        };
        let name = self.symbols.display_ident(&decl.name, self.mode);
        self.buf.push_str(kw);
        self.buf.push(' ');
        self.buf.push_str(&name);
        self.buf.push('(');
        let params = decl
            .params
            .iter()
            .map(|param| self.format_param(param))
            .collect::<Vec<_>>()
            .join(", ");
        self.buf.push_str(&params);
        self.buf.push(')');
        self.buf.push_str(match self.mode {
            Mode::Compact => "->",
            Mode::Relaxed => " -> ",
        });
        self.buf.push_str(&self.format_type_expr(&decl.ret));
        if !decl.effects.is_empty() {
            match self.mode {
                Mode::Compact => {
                    self.buf.push_str(" eff [");
                }
                Mode::Relaxed => {
                    self.buf.push_str("\n  eff [");
                }
            }
            self.buf.push_str(&decl.effects.join(", "));
            self.buf.push(']');
        }
        write_block(self, &decl.body.raw);
    }

    fn format_param(&self, param: &Param) -> String {
        let name = self.symbols.display_ident(&param.name, self.mode);
        let ty = self.format_type_expr(&param.ty);
        format!("{name}: {ty}")
    }

    fn format_type_expr(&self, expr: &TypeExpr) -> String {
        match expr {
            TypeExpr::Path(parts) => {
                let segments = parts
                    .iter()
                    .map(|p| self.symbols.display_ident(p, self.mode))
                    .collect::<Vec<_>>();
                segments.join(".")
            }
            TypeExpr::Record(fields) => {
                let inner = fields
                    .iter()
                    .map(|field| self.format_record_field(field))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {inner} }}")
            }
        }
    }

    fn format_record_field(&self, field: &RecordField) -> String {
        let name = self.symbols.display_ident(&field.name, self.mode);
        let ty = self.format_type_expr(&field.ty);
        format!("{name}: {ty}")
    }

    fn module_path(&self) -> String {
        self.module
            .path
            .as_str_vec()
            .iter()
            .map(|segment| segment.as_str())
            .collect::<Vec<_>>()
            .join(".")
    }

    fn section_break(&mut self) {
        if self.sections_emitted == 0 {
            if !self.buf.ends_with('\n') {
                self.buf.push('\n');
            }
            if matches!(self.mode, Mode::Relaxed) && !self.buf.ends_with("\n\n") {
                self.buf.push('\n');
            }
        } else {
            if !self.buf.ends_with('\n') {
                self.buf.push('\n');
            }
            self.buf.push('\n');
        }
        if matches!(self.mode, Mode::Relaxed) && !self.buf.ends_with("\n\n") {
            self.buf.push('\n');
        }
        self.sections_emitted += 1;
    }
}

struct SymbolTable {
    long_to_short: HashMap<String, String>,
    short_to_long: HashMap<String, String>,
    style: SymMapStyle,
}

impl SymbolTable {
    fn new(module: &Module, style: SymMapStyle) -> Self {
        let mut long_to_short = HashMap::new();
        let mut short_to_long = HashMap::new();
        for item in &module.items {
            if let Item::Symbol(sym) = item {
                for pair in &sym.pairs {
                    long_to_short.insert(pair.long.clone(), pair.short.clone());
                    short_to_long.insert(pair.short.clone(), pair.long.clone());
                }
            }
        }
        Self {
            long_to_short,
            short_to_long,
            style,
        }
    }

    fn display_ident(&self, ident: &str, mode: Mode) -> String {
        match mode {
            Mode::Compact => self
                .long_to_short
                .get(ident)
                .cloned()
                .unwrap_or_else(|| ident.to_string()),
            Mode::Relaxed => self
                .short_to_long
                .get(ident)
                .cloned()
                .unwrap_or_else(|| ident.to_string()),
        }
    }

    fn ordered_pairs<'a>(&'a self, pairs: &'a [SymbolPair]) -> Vec<(String, String)> {
        let mut local = pairs
            .iter()
            .map(|p| (p.long.clone(), p.short.clone()))
            .collect::<Vec<_>>();
        match self.style {
            SymMapStyle::Respect => local,
            SymMapStyle::Reflow => {
                local.sort_by(|a, b| a.0.cmp(&b.0));
                local
            }
        }
    }
}
fn write_block(formatter: &mut Formatter<'_>, raw: &str) {
    if matches!(formatter.mode, Mode::Compact) {
        formatter.buf.push(' ');
        formatter.buf.push_str(raw);
        formatter.buf.push('\n');
        return;
    }

    // For relaxed mode, reformat the block with proper indentation
    formatter.buf.push('\n');
    let trimmed = raw.trim();

    // Remove outer braces if present
    let content = if trimmed.starts_with('{') && trimmed.ends_with('}') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    formatter.buf.push_str("{\n");

    // Process each line, tracking brace depth for proper indentation
    let mut indent_level: usize = 1; // Start at 1 because we're inside the function body
    for line in content.lines() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        // Decrease indent before printing if line starts with }
        if trimmed_line.starts_with('}') {
            indent_level = indent_level.saturating_sub(1);
        }

        // Add indentation
        for _ in 0..indent_level {
            formatter.buf.push_str("  ");
        }
        formatter.buf.push_str(trimmed_line);
        formatter.buf.push('\n');

        // Increase indent after printing if line ends with {
        if trimmed_line.ends_with('{') {
            indent_level += 1;
        }
    }

    formatter.buf.push_str("}\n");
}
