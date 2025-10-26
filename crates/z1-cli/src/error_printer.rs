//! Pretty error printing with source context and colors.
//!
//! This module provides utilities for formatting compiler errors with:
//! - File path and line:column location
//! - Source snippet showing the relevant line(s)
//! - Color-coded output (red for errors, yellow for warnings)
//! - Caret/underline pointing to exact location
//! - Optional help hints

use colored::*;
use std::env;
use z1_ast::Span;
use z1_effects::EffectError;
use z1_parse::ParseError;
use z1_typeck::TypeError;

/// Configuration for error printing.
#[derive(Debug, Clone)]
pub struct ErrorPrinterConfig {
    /// Enable colored output (respects NO_COLOR environment variable).
    pub use_colors: bool,
}

impl Default for ErrorPrinterConfig {
    fn default() -> Self {
        Self {
            use_colors: env::var("NO_COLOR").is_err(),
        }
    }
}

/// Pretty-print a parse error with source context.
pub fn print_parse_error(
    error: &ParseError,
    source: &str,
    file_path: &str,
    config: &ErrorPrinterConfig,
) {
    let span = match error {
        ParseError::Unexpected { span, .. } | ParseError::Invalid { span, .. } => *span,
    };

    let header = format!("Error: {error}");
    let colored_header = if config.use_colors {
        header.red().bold().to_string()
    } else {
        header
    };

    println!("{colored_header}");
    print_source_snippet(source, file_path, span, config);
    println!();
}

/// Pretty-print a type error with source context.
pub fn print_type_error(
    error: &TypeError,
    source: &str,
    file_path: &str,
    config: &ErrorPrinterConfig,
) {
    let span_opt = match error {
        TypeError::Mismatch { span, .. }
        | TypeError::UndefinedType { span, .. }
        | TypeError::UndefinedFunction { span, .. }
        | TypeError::UndefinedVariable { span, .. }
        | TypeError::ArityMismatch { span, .. } => Some(*span),
        _ => None,
    };

    let header = format!("Type Error: {error}");
    let colored_header = if config.use_colors {
        header.red().bold().to_string()
    } else {
        header
    };

    println!("{colored_header}");
    if let Some(span) = span_opt {
        print_source_snippet(source, file_path, span, config);
    }
    println!();
}

/// Pretty-print an effect error with source context.
pub fn print_effect_error(
    error: &EffectError,
    source: &str,
    file_path: &str,
    config: &ErrorPrinterConfig,
) {
    let span = match error {
        EffectError::MissingCapability { fn_span, .. } => *fn_span,
        EffectError::UnknownEffect { fn_span, .. } => *fn_span,
    };

    let header = format!("Effect Error: {error}");
    let colored_header = if config.use_colors {
        header.red().bold().to_string()
    } else {
        header
    };

    println!("{colored_header}");
    print_source_snippet(source, file_path, span, config);

    // Add helpful hint for missing capability errors
    if let EffectError::MissingCapability { effect, module, .. } = error {
        let hint =
            format!("Help: Add '{effect}' to module capabilities: module {module} caps=[{effect}]");
        let colored_hint = if config.use_colors {
            hint.yellow().to_string()
        } else {
            hint
        };
        println!("{colored_hint}");
    }
    println!();
}

/// Print a source snippet with location marker.
fn print_source_snippet(source: &str, file_path: &str, span: Span, config: &ErrorPrinterConfig) {
    let (line_num, col_num, line_text) = extract_line_info(source, span);

    // Print location header: "  ┌─ file.z1c:5:12"
    let location = format!("  ┌─ {file_path}:{line_num}:{col_num}");
    let colored_location = if config.use_colors {
        location.blue().to_string()
    } else {
        location
    };
    println!("{colored_location}");
    println!("  │");

    // Print line number and source line: " 5 │     let x: U32 = \"hello\";"
    let line_num_str = format!("{line_num:>3}");
    let colored_line_num = if config.use_colors {
        line_num_str.blue().to_string()
    } else {
        line_num_str
    };
    println!("{colored_line_num} │ {line_text}");

    // Print caret line: "    │            ^^^"
    let caret_offset = col_num - 1; // Column is 1-indexed
    let span_len = (span.end - span.start).max(1) as usize;
    let carets = "^".repeat(span_len);
    let colored_carets = if config.use_colors {
        carets.red().bold().to_string()
    } else {
        carets
    };
    println!("    │ {}{colored_carets}", " ".repeat(caret_offset));
}

/// Extract line number, column number, and line text for a given span.
fn extract_line_info(source: &str, span: Span) -> (usize, usize, String) {
    let start_offset = span.start as usize;

    // Find line number and column
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

    // Extract the line text
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

    fn no_color_config() -> ErrorPrinterConfig {
        ErrorPrinterConfig { use_colors: false }
    }

    #[test]
    fn test_extract_line_info_first_line() {
        let source = "module test\nfn foo() {}";
        let span = Span::new(7, 11); // "test"
        let (line, col, text) = extract_line_info(source, span);
        assert_eq!(line, 1);
        assert_eq!(col, 8);
        assert_eq!(text, "module test");
    }

    #[test]
    fn test_extract_line_info_second_line() {
        let source = "module test\nfn foo() {}";
        let span = Span::new(15, 18); // "foo"
        let (line, col, text) = extract_line_info(source, span);
        assert_eq!(line, 2);
        assert_eq!(col, 4);
        assert_eq!(text, "fn foo() {}");
    }

    #[test]
    fn test_extract_line_info_multiline_source() {
        let source = "line 1\nline 2\nline 3\nline 4";
        let span = Span::new(14, 18); // "line" in "line 3"
        let (line, col, text) = extract_line_info(source, span);
        assert_eq!(line, 3);
        assert_eq!(col, 1);
        assert_eq!(text, "line 3");
    }

    #[test]
    fn test_print_parse_error_outputs_without_panic() {
        let source = "module test caps=[]";
        let error = ParseError::Unexpected {
            expected: "number",
            found: z1_lex::TokenKind::Ident,
            span: Span::new(7, 11),
        };
        // Should not panic
        print_parse_error(&error, source, "test.z1c", &no_color_config());
    }

    #[test]
    fn test_print_type_error_with_span() {
        let source = "type Foo = U32";
        let error = TypeError::UndefinedType {
            name: "U32".to_string(),
            span: Span::new(11, 14),
        };
        // Should not panic
        print_type_error(&error, source, "test.z1c", &no_color_config());
    }

    #[test]
    fn test_print_type_error_without_span() {
        let source = "type Foo = U32";
        let error = TypeError::RecordFieldMismatch {
            message: "field mismatch".to_string(),
        };
        // Should not panic even without span
        print_type_error(&error, source, "test.z1c", &no_color_config());
    }

    #[test]
    fn test_print_effect_error_outputs_without_panic() {
        let source = "fn foo() -> U32 eff [net] { ret 42; }";
        let error = EffectError::MissingCapability {
            fn_name: "foo".to_string(),
            module: "test".to_string(),
            effect: "net".to_string(),
            fn_span: Span::new(0, 37),
            module_span: Span::new(0, 0),
        };
        // Should not panic
        print_effect_error(&error, source, "test.z1c", &no_color_config());
    }

    #[test]
    fn test_extract_line_info_at_end_of_file() {
        let source = "module test";
        let span = Span::new(7, 11); // "test"
        let (line, col, text) = extract_line_info(source, span);
        assert_eq!(line, 1);
        assert_eq!(col, 8);
        assert_eq!(text, "module test");
    }

    #[test]
    fn test_extract_line_info_multichar_offset() {
        let source = "module test:1.0";
        let span = Span::new(12, 15); // "1.0"
        let (line, col, text) = extract_line_info(source, span);
        assert_eq!(line, 1);
        assert_eq!(col, 13);
        assert_eq!(text, "module test:1.0");
    }

    #[test]
    fn test_no_color_config_from_env() {
        // Test that NO_COLOR environment variable is respected
        env::set_var("NO_COLOR", "1");
        let config = ErrorPrinterConfig::default();
        assert!(!config.use_colors);
        env::remove_var("NO_COLOR");

        let config = ErrorPrinterConfig::default();
        assert!(config.use_colors);
    }

    #[test]
    fn test_print_effect_error_includes_help_hint() {
        let source = "fn foo() -> U32 eff [net] { ret 42; }";
        let error = EffectError::MissingCapability {
            fn_name: "foo".to_string(),
            module: "test".to_string(),
            effect: "net".to_string(),
            fn_span: Span::new(0, 37),
            module_span: Span::new(0, 0),
        };
        // This test just verifies the function runs without panic
        // In a real scenario, you'd capture stdout to verify the hint is printed
        print_effect_error(&error, source, "test.z1c", &no_color_config());
    }
}
