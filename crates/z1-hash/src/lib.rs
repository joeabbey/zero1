use sha3::{Digest, Sha3_256};
use z1_ast::{
    Block, FnDecl, Import, Item, Module, Param, RecordField, SymbolMap, TypeDecl, TypeExpr,
};

type HashState = Sha3_256;

/// Container for both semantic and format hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleHashes {
    pub semantic: String,
    pub format: String,
}

pub fn module_hashes(module: &Module) -> ModuleHashes {
    ModuleHashes {
        semantic: hash_module(module, false),
        format: hash_module(module, true),
    }
}

fn hash_module(module: &Module, include_symbol_map: bool) -> String {
    let mut hasher = Sha3_256::new();
    feed_str(&mut hasher, "module");
    for segment in module.path.as_str_vec() {
        feed_str(&mut hasher, segment);
    }
    feed_opt_str(&mut hasher, module.version.as_deref());
    match module.ctx_budget {
        Some(value) => {
            hasher.update([1]);
            feed_u32(&mut hasher, value);
        }
        None => {
            hasher.update([0]);
        }
    }
    hasher.update((module.caps.len() as u32).to_le_bytes());
    for cap in &module.caps {
        feed_str(&mut hasher, cap);
    }
    for item in &module.items {
        hash_item(&mut hasher, item, include_symbol_map);
    }
    let digest = hasher.finalize();
    format!("{digest:x}")
}

fn hash_item(hasher: &mut HashState, item: &Item, include_symbol_map: bool) {
    match item {
        Item::Import(import) => {
            feed_str(hasher, "import");
            hash_import(hasher, import);
        }
        Item::Symbol(symbols) => {
            if include_symbol_map {
                feed_str(hasher, "symbol_map");
                hash_symbol_map(hasher, symbols);
            }
        }
        Item::Type(ty) => {
            feed_str(hasher, "type");
            hash_type_decl(hasher, ty);
        }
        Item::Fn(func) => {
            feed_str(hasher, "fn");
            hash_fn_decl(hasher, func);
        }
    }
}

fn hash_import(hasher: &mut HashState, import: &Import) {
    feed_str(hasher, &import.path);
    feed_opt_str(hasher, import.alias.as_deref());
    hasher.update((import.only.len() as u32).to_le_bytes());
    for ident in &import.only {
        feed_str(hasher, ident);
    }
}

fn hash_symbol_map(hasher: &mut HashState, symbols: &SymbolMap) {
    hasher.update((symbols.pairs.len() as u32).to_le_bytes());
    for pair in &symbols.pairs {
        feed_str(hasher, &pair.long);
        feed_str(hasher, &pair.short);
    }
}

fn hash_type_decl(hasher: &mut HashState, ty: &TypeDecl) {
    feed_str(hasher, &ty.name);
    hash_type_expr(hasher, &ty.expr);
}

fn hash_type_expr(hasher: &mut HashState, expr: &TypeExpr) {
    match expr {
        TypeExpr::Path(segments) => {
            feed_str(hasher, "path");
            hasher.update((segments.len() as u32).to_le_bytes());
            for segment in segments {
                feed_str(hasher, segment);
            }
        }
        TypeExpr::Record(fields) => {
            feed_str(hasher, "record");
            hasher.update((fields.len() as u32).to_le_bytes());
            for field in fields {
                hash_record_field(hasher, field);
            }
        }
    }
}

fn hash_record_field(hasher: &mut HashState, field: &RecordField) {
    feed_str(hasher, &field.name);
    hash_type_expr(hasher, &field.ty);
}

fn hash_fn_decl(hasher: &mut HashState, func: &FnDecl) {
    feed_str(hasher, &func.name);
    hasher.update((func.params.len() as u32).to_le_bytes());
    for param in &func.params {
        hash_param(hasher, param);
    }
    hash_type_expr(hasher, &func.ret);
    hasher.update((func.effects.len() as u32).to_le_bytes());
    for eff in &func.effects {
        feed_str(hasher, eff);
    }
    hash_block(hasher, &func.body);
}

fn hash_param(hasher: &mut HashState, param: &Param) {
    feed_str(hasher, &param.name);
    hash_type_expr(hasher, &param.ty);
}

fn hash_block(hasher: &mut HashState, block: &Block) {
    feed_str(hasher, &block.raw);
}

fn feed_str(hasher: &mut HashState, value: &str) {
    hasher.update(value.as_bytes());
    hasher.update([0]);
}

fn feed_opt_str(hasher: &mut HashState, value: Option<&str>) {
    match value {
        Some(val) => {
            hasher.update([1]);
            feed_str(hasher, val);
        }
        None => {
            hasher.update([0]);
        }
    }
}

fn feed_u32(hasher: &mut HashState, value: u32) {
    hasher.update(value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_hash_changes_when_symbol_map_differs() {
        let source = include_str!("../../../fixtures/cells/http_server.z1c");
        let module = z1_parse::parse_module(source).expect("parse");
        let hashes = module_hashes(&module);

        let mut modified = module.clone();
        if let Some(Item::Symbol(symbols)) = modified
            .items
            .iter_mut()
            .find(|item| matches!(item, Item::Symbol(_)))
        {
            symbols.pairs[0].short.push('x');
        }
        let hashes_modified = module_hashes(&modified);
        assert_eq!(hashes.semantic, hashes_modified.semantic);
        assert_ne!(hashes.format, hashes_modified.format);
    }

    #[test]
    fn semantic_hash_changes_on_body_edits() {
        let source = include_str!("../../../fixtures/cells/http_server.z1c");
        let mut module = z1_parse::parse_module(source).expect("parse");
        let hashes = module_hashes(&module);
        if let Some(Item::Fn(func)) = module
            .items
            .iter_mut()
            .find(|item| matches!(item, Item::Fn(_)))
        {
            func.body.raw.push_str("// change");
        }
        let hashes_modified = module_hashes(&module);
        assert_ne!(hashes.semantic, hashes_modified.semantic);
        assert_ne!(hashes.format, hashes_modified.format);
    }
}
