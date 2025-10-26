//! Comprehensive diagnostic system with warnings, suggestions, and multi-error reporting.
//!
//! This module provides:
//! - Diagnostic levels: Error, Warning, Info, Help
//! - Diagnostic collection across multiple checkers
//! - Suggestion system with fuzzy name matching
//! - JSON output for tooling integration

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::should_implement_trait)]
#![allow(dead_code)]
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use z1_ast::Span;
use z1_effects::EffectError;
use z1_parse::ParseError;
use z1_typeck::TypeError;

/// Diagnostic severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
    Help,
}

impl DiagnosticLevel {
    /// Get the symbol/icon for this diagnostic level.
    pub fn symbol(&self) -> &'static str {
        match self {
            DiagnosticLevel::Error => "✗",
            DiagnosticLevel::Warning => "⚠",
            DiagnosticLevel::Info => "ℹ",
            DiagnosticLevel::Help => "💡",
        }
    }

    /// Get the color for this diagnostic level.
    pub fn color(&self, text: &str) -> String {
        match self {
            DiagnosticLevel::Error => text.red().bold().to_string(),
            DiagnosticLevel::Warning => text.yellow().to_string(),
            DiagnosticLevel::Info => text.cyan().to_string(),
            DiagnosticLevel::Help => text.green().to_string(),
        }
    }
}

/// A single diagnostic message (error, warning, info, or help).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: DiagnosticLevel,
    pub message: String,
    pub span: Option<Span>,
    pub source_file: String,
    pub suggestion: Option<String>,
    pub code: Option<String>,
}

impl Diagnostic {
    /// Create a new error diagnostic.
    pub fn error(message: String, source_file: String) -> Self {
        Self {
            level: DiagnosticLevel::Error,
            message,
            span: None,
            source_file,
            suggestion: None,
            code: None,
        }
    }

    /// Create a new warning diagnostic.
    pub fn warning(message: String, source_file: String) -> Self {
        Self {
            level: DiagnosticLevel::Warning,
            message,
            span: None,
            source_file,
            suggestion: None,
            code: None,
        }
    }

    /// Create a new info diagnostic.
    pub fn info(message: String, source_file: String) -> Self {
        Self {
            level: DiagnosticLevel::Info,
            message,
            span: None,
            source_file,
            suggestion: None,
            code: None,
        }
    }

    /// Set the span for this diagnostic.
    pub fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }

    /// Set a suggestion for this diagnostic.
    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }

    /// Set an error code for this diagnostic.
    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    /// Convert a ParseError to a Diagnostic.
    pub fn from_parse_error(error: &ParseError, source_file: String) -> Self {
        let span = match error {
            ParseError::Unexpected { span, .. } | ParseError::Invalid { span, .. } => *span,
        };

        Self::error(format!("Parse Error: {error}"), source_file)
            .with_span(span)
            .with_code("P001".to_string())
    }

    /// Convert a TypeError to a Diagnostic.
    pub fn from_type_error(error: &TypeError, source_file: String) -> Self {
        let span_opt = match error {
            TypeError::Mismatch { span, .. }
            | TypeError::UndefinedType { span, .. }
            | TypeError::UndefinedFunction { span, .. }
            | TypeError::UndefinedVariable { span, .. }
            | TypeError::ArityMismatch { span, .. } => Some(*span),
            _ => None,
        };

        let mut diag =
            Self::error(format!("Type Error: {error}"), source_file).with_code("T001".to_string());

        if let Some(span) = span_opt {
            diag = diag.with_span(span);
        }

        diag
    }

    /// Convert an EffectError to a Diagnostic with suggestion.
    pub fn from_effect_error(error: &EffectError, source_file: String) -> Self {
        let (span, suggestion) = match error {
            EffectError::MissingCapability {
                fn_span,
                effect,
                module,
                ..
            } => {
                let suggestion = format!(
                    "Add '{effect}' to module capabilities: module {module} caps=[{effect}]"
                );
                (*fn_span, Some(suggestion))
            }
            EffectError::UnknownEffect { fn_span, .. } => (*fn_span, None),
        };

        let mut diag = Self::error(format!("Effect Error: {error}"), source_file)
            .with_span(span)
            .with_code("E001".to_string());

        if let Some(s) = suggestion {
            diag = diag.with_suggestion(s);
        }

        diag
    }
}

/// Configuration for diagnostic output.
#[derive(Debug, Clone)]
pub struct DiagnosticConfig {
    /// Enable colored output.
    pub use_colors: bool,
    /// Warning level: all, default, none.
    pub warn_level: WarnLevel,
    /// Treat warnings as errors.
    pub warn_as_error: bool,
    /// Maximum number of errors before stopping.
    pub max_errors: usize,
    /// Output as JSON.
    pub json_output: bool,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            use_colors: std::env::var("NO_COLOR").is_err(),
            warn_level: WarnLevel::Default,
            warn_as_error: false,
            max_errors: 50,
            json_output: false,
        }
    }
}

/// Warning level configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarnLevel {
    All,
    Default,
    None,
}

impl WarnLevel {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "all" => Some(WarnLevel::All),
            "default" => Some(WarnLevel::Default),
            "none" => Some(WarnLevel::None),
            _ => None,
        }
    }
}

/// Collects diagnostics from multiple sources.
#[derive(Debug, Default)]
pub struct DiagnosticCollector {
    diagnostics: Vec<Diagnostic>,
    error_count: usize,
    warning_count: usize,
}

impl DiagnosticCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a diagnostic to the collection.
    pub fn add(&mut self, diagnostic: Diagnostic) {
        match diagnostic.level {
            DiagnosticLevel::Error => self.error_count += 1,
            DiagnosticLevel::Warning => self.warning_count += 1,
            _ => {}
        }
        self.diagnostics.push(diagnostic);
    }

    /// Add an error diagnostic.
    pub fn add_error(&mut self, message: String, source_file: String) {
        self.add(Diagnostic::error(message, source_file));
    }

    /// Add a warning diagnostic.
    pub fn add_warning(&mut self, message: String, source_file: String) {
        self.add(Diagnostic::warning(message, source_file));
    }

    /// Add an info diagnostic.
    pub fn add_info(&mut self, message: String, source_file: String) {
        self.add(Diagnostic::info(message, source_file));
    }

    /// Get all diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Get error count.
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Get warning count.
    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    /// Check if there are any errors.
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Filter diagnostics by level.
    pub fn filter_by_level(&self, level: DiagnosticLevel) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.level == level)
            .collect()
    }

    /// Group diagnostics by file.
    pub fn by_file(&self) -> HashMap<String, Vec<&Diagnostic>> {
        let mut map: HashMap<String, Vec<&Diagnostic>> = HashMap::new();
        for diag in &self.diagnostics {
            map.entry(diag.source_file.clone()).or_default().push(diag);
        }
        map
    }
}

/// Compute Levenshtein distance between two strings for fuzzy matching.
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

/// Suggest a similar name from available names using fuzzy matching.
///
/// Returns the most similar name within edit distance of 2, or None.
pub fn suggest_similar_name(typo: &str, available: &[String]) -> Option<String> {
    available
        .iter()
        .filter(|name| levenshtein_distance(typo, name) <= 2)
        .min_by_key(|name| levenshtein_distance(typo, name))
        .cloned()
}

/// Print diagnostics to stderr with pretty formatting.
pub fn print_diagnostics(diagnostics: &[Diagnostic], source: &str, config: &DiagnosticConfig) {
    if config.json_output {
        print_diagnostics_json(diagnostics);
        return;
    }

    for diag in diagnostics {
        print_diagnostic(diag, source, config);
    }

    // Print summary
    let error_count = diagnostics
        .iter()
        .filter(|d| matches!(d.level, DiagnosticLevel::Error))
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| matches!(d.level, DiagnosticLevel::Warning))
        .count();

    if error_count > 0 || warning_count > 0 {
        eprintln!();
        let summary = format!("{error_count} error(s), {warning_count} warning(s)");
        if config.use_colors {
            eprintln!("{}", summary.bold());
        } else {
            eprintln!("{summary}");
        }
    }
}

/// Print a single diagnostic with pretty formatting.
pub fn print_diagnostic(diagnostic: &Diagnostic, source: &str, config: &DiagnosticConfig) {
    let symbol = diagnostic.level.symbol();
    let header = format!("{} {}", symbol, diagnostic.message);

    let colored_header = if config.use_colors {
        diagnostic.level.color(&header)
    } else {
        header
    };

    eprintln!("{colored_header}");

    if let Some(span) = diagnostic.span {
        print_source_snippet(source, &diagnostic.source_file, span, config);
    }

    if let Some(suggestion) = &diagnostic.suggestion {
        let help_msg = format!("💡 Help: {suggestion}");
        let colored_help = if config.use_colors {
            help_msg.green().to_string()
        } else {
            help_msg
        };
        eprintln!("{colored_help}");
    }

    eprintln!();
}

/// Print diagnostics as JSON.
fn print_diagnostics_json(diagnostics: &[Diagnostic]) {
    let json = serde_json::to_string_pretty(diagnostics)
        .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize diagnostics: {e}\"}}"));
    println!("{json}");
}

/// Print a source snippet with location marker.
fn print_source_snippet(source: &str, file_path: &str, span: Span, config: &DiagnosticConfig) {
    let (line_num, col_num, line_text) = extract_line_info(source, span);

    let location = format!("  ┌─ {file_path}:{line_num}:{col_num}");
    let colored_location = if config.use_colors {
        location.blue().to_string()
    } else {
        location
    };
    eprintln!("{colored_location}");
    eprintln!("  │");

    let line_num_str = format!("{line_num:>3}");
    let colored_line_num = if config.use_colors {
        line_num_str.blue().to_string()
    } else {
        line_num_str
    };
    eprintln!("{colored_line_num} │ {line_text}");

    let caret_offset = col_num - 1;
    let span_len = (span.end - span.start).max(1) as usize;
    let carets = "^".repeat(span_len);
    let colored_carets = if config.use_colors {
        carets.red().bold().to_string()
    } else {
        carets
    };
    eprintln!("    │ {}{}", " ".repeat(caret_offset), colored_carets);
}

/// Extract line number, column number, and line text for a given span.
fn extract_line_info(source: &str, span: Span) -> (usize, usize, String) {
    let start_offset = span.start as usize;

    let mut line_num = 1;
    let mut col_num = 1;
    let mut line_start_offset = 0;

    for (offset, ch) in source.char_indices() {
        if offset == start_offset {
            break;
        }
        if ch == '\n' {
            line_num += 1;
            col_num = 1;
            line_start_offset = offset + 1;
        } else {
            col_num += 1;
        }
    }

    let line_end_offset = source[line_start_offset..]
        .find('\n')
        .map(|pos| line_start_offset + pos)
        .unwrap_or(source.len());

    let line_text = source[line_start_offset..line_end_offset].to_string();

    (line_num, col_num, line_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance_identical() {
        assert_eq!(levenshtein_distance("hello", "hello"), 0);
    }

    #[test]
    fn test_levenshtein_distance_one_edit() {
        assert_eq!(levenshtein_distance("hello", "helo"), 1);
        assert_eq!(levenshtein_distance("hello", "helloo"), 1);
        assert_eq!(levenshtein_distance("hello", "hallo"), 1);
    }

    #[test]
    fn test_levenshtein_distance_multiple_edits() {
        assert_eq!(levenshtein_distance("hello", "halo"), 2);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_suggest_similar_name_exact_match() {
        let available = vec!["hello".to_string(), "world".to_string()];
        assert_eq!(
            suggest_similar_name("hello", &available),
            Some("hello".to_string())
        );
    }

    #[test]
    fn test_suggest_similar_name_close_match() {
        let available = vec!["handler".to_string(), "helper".to_string()];
        assert_eq!(
            suggest_similar_name("hanler", &available),
            Some("handler".to_string())
        );
    }

    #[test]
    fn test_suggest_similar_name_no_match() {
        let available = vec!["foo".to_string(), "bar".to_string()];
        assert_eq!(
            suggest_similar_name("completely_different", &available),
            None
        );
    }

    #[test]
    fn test_diagnostic_collector_counts() {
        let mut collector = DiagnosticCollector::new();
        collector.add_error("error1".to_string(), "test.z1c".to_string());
        collector.add_error("error2".to_string(), "test.z1c".to_string());
        collector.add_warning("warning1".to_string(), "test.z1c".to_string());

        assert_eq!(collector.error_count(), 2);
        assert_eq!(collector.warning_count(), 1);
        assert!(collector.has_errors());
    }

    #[test]
    fn test_diagnostic_collector_filter_by_level() {
        let mut collector = DiagnosticCollector::new();
        collector.add_error("error1".to_string(), "test.z1c".to_string());
        collector.add_warning("warning1".to_string(), "test.z1c".to_string());

        let errors = collector.filter_by_level(DiagnosticLevel::Error);
        assert_eq!(errors.len(), 1);

        let warnings = collector.filter_by_level(DiagnosticLevel::Warning);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn test_diagnostic_from_parse_error() {
        let error = ParseError::Unexpected {
            expected: "number",
            found: z1_lex::TokenKind::Ident,
            span: Span::new(5, 10),
        };

        let diag = Diagnostic::from_parse_error(&error, "test.z1c".to_string());
        assert_eq!(diag.level, DiagnosticLevel::Error);
        assert!(diag.span.is_some());
        assert_eq!(diag.code.as_deref(), Some("P001"));
    }

    #[test]
    fn test_diagnostic_from_effect_error_with_suggestion() {
        let error = EffectError::MissingCapability {
            fn_name: "foo".to_string(),
            module: "test".to_string(),
            effect: "net".to_string(),
            fn_span: Span::new(0, 10),
            module_span: Span::new(0, 5),
        };

        let diag = Diagnostic::from_effect_error(&error, "test.z1c".to_string());
        assert_eq!(diag.level, DiagnosticLevel::Error);
        assert!(diag.suggestion.is_some());
        assert!(diag.suggestion.unwrap().contains("caps=[net]"));
    }

    #[test]
    fn test_warn_level_from_str() {
        assert_eq!(WarnLevel::from_str("all"), Some(WarnLevel::All));
        assert_eq!(WarnLevel::from_str("default"), Some(WarnLevel::Default));
        assert_eq!(WarnLevel::from_str("none"), Some(WarnLevel::None));
        assert_eq!(WarnLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_diagnostic_collector_group_by_file() {
        let mut collector = DiagnosticCollector::new();
        collector.add_error("error1".to_string(), "file1.z1c".to_string());
        collector.add_error("error2".to_string(), "file2.z1c".to_string());
        collector.add_warning("warning1".to_string(), "file1.z1c".to_string());

        let by_file = collector.by_file();
        assert_eq!(by_file.len(), 2);
        assert_eq!(by_file.get("file1.z1c").unwrap().len(), 2);
        assert_eq!(by_file.get("file2.z1c").unwrap().len(), 1);
    }
}
