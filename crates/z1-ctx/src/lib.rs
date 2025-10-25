//! Context estimator for Zero1 cells and functions.
//!
//! This crate provides token usage estimation for Z1 modules (cells) and functions,
//! enabling enforcement of context budget limits defined in module headers.
//!
//! ## Token Cost Model
//!
//! The MVP uses a naive heuristic: `tokens ≈ ceil(chars / 3.8)`.
//! Future versions will support model-specific dictionaries (SDict) for improved estimation.
//!
//! ## Usage
//!
//! ```rust
//! use z1_ctx::estimate_cell;
//! use z1_parse::parse_module;
//!
//! let source = r#"
//! m http.server:1.0 ctx=128 caps=[net]
//! f handler()->Unit eff [pure] { ret Unit }
//! "#;
//!
//! let module = parse_module(source).unwrap();
//! let estimate = estimate_cell(&module).unwrap();
//! println!("Estimated tokens: {}", estimate.total_tokens);
//! ```

use std::fmt;
use thiserror::Error;
use z1_ast::{FnDecl, Module, Span};
use z1_fmt::{format_module, FmtOptions, Mode};

/// Default token cost model: tokens ≈ ceil(chars / 3.8)
pub const DEFAULT_CHARS_PER_TOKEN: f64 = 3.8;

/// Errors that can occur during context estimation.
#[derive(Debug, Error)]
pub enum CtxError {
    #[error("formatting error during estimation: {0}")]
    Format(#[from] z1_fmt::FmtError),

    #[error("cell exceeds context budget: {actual}/{budget} tokens. {suggestion}")]
    BudgetExceeded {
        actual: u32,
        budget: u32,
        suggestion: String,
        span: Span,
    },

    #[error("function '{name}' exceeds context budget: {actual}/{budget} tokens")]
    FnBudgetExceeded {
        name: String,
        actual: u32,
        budget: u32,
        span: Span,
    },
}

/// Context estimation result for a cell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellEstimate {
    /// Total estimated tokens for the entire cell
    pub total_tokens: u32,
    /// Declared context budget from module header
    pub budget: Option<u32>,
    /// Per-function estimates
    pub functions: Vec<FnEstimate>,
    /// Character count of compact representation
    pub char_count: usize,
}

/// Context estimation result for a single function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnEstimate {
    /// Function name
    pub name: String,
    /// Estimated tokens for this function
    pub tokens: u32,
    /// Character count for this function
    pub chars: usize,
    /// Source span
    pub span: Span,
}

/// Configuration for context estimation.
#[derive(Debug, Clone)]
pub struct EstimateConfig {
    /// Characters per token ratio (default: 3.8)
    pub chars_per_token: f64,
    /// Whether to enforce budget limits
    pub enforce_budget: bool,
}

impl Default for EstimateConfig {
    fn default() -> Self {
        Self {
            chars_per_token: DEFAULT_CHARS_PER_TOKEN,
            enforce_budget: true,
        }
    }
}

/// Estimates token usage for a module using default configuration.
///
/// This function:
/// 1. Formats the AST to compact text using the current SymbolMap
/// 2. Counts characters and estimates tokens via naive heuristic
/// 3. Validates against declared budget (if `ctx=N` is present)
/// 4. Returns error if budget is exceeded, or Ok with estimate
///
/// # Errors
///
/// Returns `CtxError::BudgetExceeded` if the module declares a context budget
/// and the estimated tokens exceed it.
pub fn estimate_cell(module: &Module) -> Result<CellEstimate, CtxError> {
    estimate_cell_with_config(module, &EstimateConfig::default())
}

/// Estimates token usage for a module with custom configuration.
pub fn estimate_cell_with_config(
    module: &Module,
    config: &EstimateConfig,
) -> Result<CellEstimate, CtxError> {
    // Format to compact mode for token estimation
    let compact_text = format_module(module, Mode::Compact, &FmtOptions::default())?;
    let char_count = compact_text.len();

    // Calculate total tokens using naive heuristic
    let total_tokens = estimate_tokens_from_chars(char_count, config.chars_per_token);

    // Estimate per-function tokens (approximate by line counting)
    let functions = estimate_functions(module, config);

    let estimate = CellEstimate {
        total_tokens,
        budget: module.ctx_budget,
        functions,
        char_count,
    };

    // Validate budget if enforcement is enabled
    if config.enforce_budget {
        if let Some(budget) = module.ctx_budget {
            if total_tokens > budget {
                return Err(CtxError::BudgetExceeded {
                    actual: total_tokens,
                    budget,
                    suggestion: suggest_split(&estimate),
                    span: module.span,
                });
            }
        }
    }

    Ok(estimate)
}

/// Estimates tokens from character count using the configured ratio.
fn estimate_tokens_from_chars(chars: usize, chars_per_token: f64) -> u32 {
    (chars as f64 / chars_per_token).ceil() as u32
}

/// Estimates token usage for individual functions.
///
/// Note: This is approximate as we don't have full statement-level formatting yet.
/// We estimate based on the size of the function body as a proportion of total content.
fn estimate_functions(module: &Module, config: &EstimateConfig) -> Vec<FnEstimate> {
    let mut estimates = Vec::new();

    for item in &module.items {
        if let z1_ast::Item::Fn(fn_decl) = item {
            let fn_estimate = estimate_function(fn_decl, config);
            estimates.push(fn_estimate);
        }
    }

    estimates
}

/// Estimates tokens for a single function.
fn estimate_function(fn_decl: &FnDecl, config: &EstimateConfig) -> FnEstimate {
    // Rough approximation: use body raw text length as basis
    // In future, we could format individual functions to compact mode
    let body_len = fn_decl.body.raw.len();

    // Add signature overhead (name + params + return type + effects)
    let sig_overhead = fn_decl.name.len()
        + fn_decl
            .params
            .iter()
            .map(|p| p.name.len() + 5)
            .sum::<usize>()
        + 20; // rough overhead for syntax

    let total_chars = body_len + sig_overhead;
    let tokens = estimate_tokens_from_chars(total_chars, config.chars_per_token);

    FnEstimate {
        name: fn_decl.name.clone(),
        tokens,
        chars: total_chars,
        span: fn_decl.span,
    }
}

/// Suggests how to split a cell that exceeds its budget.
fn suggest_split(estimate: &CellEstimate) -> String {
    if estimate.functions.is_empty() {
        return "Consider reducing the size of this cell.".to_string();
    }

    // Find largest function by token count
    let largest_fn = estimate.functions.iter().max_by_key(|f| f.tokens).unwrap();

    if estimate.functions.len() == 1 {
        format!(
            "Consider splitting function '{}' ({} tokens) into smaller functions.",
            largest_fn.name, largest_fn.tokens
        )
    } else {
        format!(
            "Consider moving function '{}' ({} tokens) to a separate cell.",
            largest_fn.name, largest_fn.tokens
        )
    }
}

impl fmt::Display for CellEstimate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Cell Estimate:")?;
        writeln!(f, "  Total tokens: {}", self.total_tokens)?;
        if let Some(budget) = self.budget {
            writeln!(f, "  Budget: {budget}")?;
            let percentage = (self.total_tokens as f64 / budget as f64) * 100.0;
            writeln!(f, "  Usage: {percentage:.1}%")?;
        }
        writeln!(f, "  Characters: {}", self.char_count)?;

        if !self.functions.is_empty() {
            writeln!(f, "\nFunction Estimates:")?;
            for fn_est in &self.functions {
                writeln!(
                    f,
                    "  - {}: {} tokens ({} chars)",
                    fn_est.name, fn_est.tokens, fn_est.chars
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens_from_chars() {
        // Default ratio: 3.8 chars per token
        assert_eq!(estimate_tokens_from_chars(0, DEFAULT_CHARS_PER_TOKEN), 0);
        assert_eq!(estimate_tokens_from_chars(3, DEFAULT_CHARS_PER_TOKEN), 1);
        assert_eq!(estimate_tokens_from_chars(4, DEFAULT_CHARS_PER_TOKEN), 2);
        assert_eq!(estimate_tokens_from_chars(38, DEFAULT_CHARS_PER_TOKEN), 10);
        assert_eq!(estimate_tokens_from_chars(100, DEFAULT_CHARS_PER_TOKEN), 27);
    }

    #[test]
    fn test_estimate_tokens_custom_ratio() {
        // Custom ratio: 4.0 chars per token
        assert_eq!(estimate_tokens_from_chars(100, 4.0), 25);
        assert_eq!(estimate_tokens_from_chars(17, 4.0), 5);
    }
}
