//! Integration tests for the diagnostic system.
//!
//! Tests warning detection, suggestion system, multi-error reporting, and output formatting.

use z1_ast::Span;
use z1_cli::diagnostics::{
    levenshtein_distance, print_diagnostics, suggest_similar_name, Diagnostic, DiagnosticCollector,
    DiagnosticConfig, DiagnosticLevel, WarnLevel,
};

#[test]
fn test_diagnostic_collector_counts_errors_and_warnings() {
    let mut collector = DiagnosticCollector::new();

    collector.add_error("error1".to_string(), "test.z1c".to_string());
    collector.add_error("error2".to_string(), "test.z1c".to_string());
    collector.add_warning("warning1".to_string(), "test.z1c".to_string());
    collector.add_warning("warning2".to_string(), "test.z1c".to_string());
    collector.add_warning("warning3".to_string(), "test.z1c".to_string());

    assert_eq!(collector.error_count(), 2);
    assert_eq!(collector.warning_count(), 3);
    assert!(collector.has_errors());
}

#[test]
fn test_diagnostic_collector_filter_by_level() {
    let mut collector = DiagnosticCollector::new();

    collector.add_error("error1".to_string(), "test.z1c".to_string());
    collector.add_error("error2".to_string(), "test.z1c".to_string());
    collector.add_warning("warning1".to_string(), "test.z1c".to_string());
    collector.add_info("info1".to_string(), "test.z1c".to_string());

    let errors = collector.filter_by_level(DiagnosticLevel::Error);
    assert_eq!(errors.len(), 2);

    let warnings = collector.filter_by_level(DiagnosticLevel::Warning);
    assert_eq!(warnings.len(), 1);

    let infos = collector.filter_by_level(DiagnosticLevel::Info);
    assert_eq!(infos.len(), 1);
}

#[test]
fn test_diagnostic_collector_group_by_file() {
    let mut collector = DiagnosticCollector::new();

    collector.add_error("error1".to_string(), "file1.z1c".to_string());
    collector.add_error("error2".to_string(), "file2.z1c".to_string());
    collector.add_warning("warning1".to_string(), "file1.z1c".to_string());
    collector.add_warning("warning2".to_string(), "file1.z1c".to_string());
    collector.add_info("info1".to_string(), "file2.z1c".to_string());

    let by_file = collector.by_file();

    assert_eq!(by_file.len(), 2);
    assert_eq!(by_file.get("file1.z1c").unwrap().len(), 3); // 1 error + 2 warnings
    assert_eq!(by_file.get("file2.z1c").unwrap().len(), 2); // 1 error + 1 info
}

#[test]
fn test_levenshtein_distance_calculations() {
    // Identical strings
    assert_eq!(levenshtein_distance("hello", "hello"), 0);

    // One insertion
    assert_eq!(levenshtein_distance("hello", "helloo"), 1);

    // One deletion
    assert_eq!(levenshtein_distance("hello", "helo"), 1);

    // One substitution
    assert_eq!(levenshtein_distance("hello", "hallo"), 1);

    // Multiple edits
    assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
}

#[test]
fn test_suggest_similar_name_finds_close_matches() {
    let available = vec![
        "handler".to_string(),
        "helper".to_string(),
        "holder".to_string(),
    ];

    // Exact match (distance 0)
    assert_eq!(
        suggest_similar_name("handler", &available),
        Some("handler".to_string())
    );

    // One character off (distance 1)
    assert_eq!(
        suggest_similar_name("hanler", &available),
        Some("handler".to_string())
    );

    // Two characters off (distance 2)
    assert_eq!(
        suggest_similar_name("handr", &available),
        Some("handler".to_string())
    );
}

#[test]
fn test_suggest_similar_name_returns_none_for_distant_matches() {
    let available = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];

    // More than 2 edits away from any candidate
    assert_eq!(
        suggest_similar_name("completely_different", &available),
        None
    );
    assert_eq!(suggest_similar_name("xyz", &available), None);
}

#[test]
fn test_diagnostic_from_parse_error() {
    use z1_lex::TokenKind;
    use z1_parse::ParseError;

    let error = ParseError::Unexpected {
        expected: "number",
        found: TokenKind::Ident,
        span: Span::new(5, 10),
    };

    let diag = Diagnostic::from_parse_error(&error, "test.z1c".to_string());

    assert_eq!(diag.level, DiagnosticLevel::Error);
    assert!(diag.span.is_some());
    assert_eq!(diag.code.as_deref(), Some("P001"));
    assert!(diag.message.contains("Parse Error"));
}

#[test]
fn test_diagnostic_from_type_error_with_span() {
    use z1_typeck::TypeError;

    let error = TypeError::Mismatch {
        expected: "U32".to_string(),
        found: "Str".to_string(),
        span: Span::new(10, 15),
    };

    let diag = Diagnostic::from_type_error(&error, "test.z1c".to_string());

    assert_eq!(diag.level, DiagnosticLevel::Error);
    assert!(diag.span.is_some());
    assert_eq!(diag.code.as_deref(), Some("T001"));
    assert!(diag.message.contains("Type Error"));
}

#[test]
fn test_diagnostic_from_effect_error_includes_suggestion() {
    use z1_effects::EffectError;

    let error = EffectError::MissingCapability {
        fn_name: "foo".to_string(),
        module: "test".to_string(),
        effect: "net".to_string(),
        fn_span: Span::new(0, 10),
        module_span: Span::new(0, 5),
    };

    let diag = Diagnostic::from_effect_error(&error, "test.z1c".to_string());

    assert_eq!(diag.level, DiagnosticLevel::Error);
    assert!(diag.span.is_some());
    assert_eq!(diag.code.as_deref(), Some("E001"));
    assert!(diag.suggestion.is_some());

    let suggestion = diag.suggestion.unwrap();
    assert!(suggestion.contains("caps=[net]"));
}

#[test]
fn test_diagnostic_config_warn_level_parsing() {
    assert_eq!(WarnLevel::from_str("all"), Some(WarnLevel::All));
    assert_eq!(WarnLevel::from_str("default"), Some(WarnLevel::Default));
    assert_eq!(WarnLevel::from_str("none"), Some(WarnLevel::None));
    assert_eq!(WarnLevel::from_str("invalid"), None);
}

#[test]
fn test_diagnostic_config_respects_no_color_env() {
    // Save original env state
    let original = std::env::var("NO_COLOR").ok();

    // Test with NO_COLOR set
    std::env::set_var("NO_COLOR", "1");
    let config = DiagnosticConfig::default();
    assert!(!config.use_colors);

    // Test with NO_COLOR unset
    std::env::remove_var("NO_COLOR");
    let config = DiagnosticConfig::default();
    assert!(config.use_colors);

    // Restore original env state
    if let Some(val) = original {
        std::env::set_var("NO_COLOR", val);
    } else {
        std::env::remove_var("NO_COLOR");
    }
}

#[test]
fn test_print_diagnostics_formats_correctly() {
    let diagnostics = vec![
        Diagnostic::error("Test error".to_string(), "test.z1c".to_string())
            .with_span(Span::new(0, 5))
            .with_code("E001".to_string()),
        Diagnostic::warning("Test warning".to_string(), "test.z1c".to_string())
            .with_span(Span::new(10, 15))
            .with_suggestion("Try fixing this".to_string()),
    ];

    let source = "module test\nfn foo() {}";
    let config = DiagnosticConfig {
        use_colors: false,
        warn_level: WarnLevel::Default,
        warn_as_error: false,
        max_errors: 50,
        json_output: false,
    };

    // This test verifies the function doesn't panic
    // In a real scenario, you'd capture stderr to verify output format
    print_diagnostics(&diagnostics, source, &config);
}

#[test]
fn test_diagnostic_with_builder_pattern() {
    let diag = Diagnostic::error("Test error".to_string(), "test.z1c".to_string())
        .with_span(Span::new(0, 5))
        .with_code("E001".to_string())
        .with_suggestion("Try this fix".to_string());

    assert_eq!(diag.level, DiagnosticLevel::Error);
    assert!(diag.span.is_some());
    assert_eq!(diag.code.as_deref(), Some("E001"));
    assert_eq!(diag.suggestion.as_deref(), Some("Try this fix"));
}

#[test]
fn test_multi_error_collection_continues_after_errors() {
    // Simulate collecting multiple errors without fail-fast behavior
    let mut collector = DiagnosticCollector::new();

    // Add errors from different sources
    collector.add_error("Parse error in line 1".to_string(), "test.z1c".to_string());
    collector.add_error("Type error in line 5".to_string(), "test.z1c".to_string());
    collector.add_error(
        "Effect error in line 10".to_string(),
        "test.z1c".to_string(),
    );
    collector.add_warning(
        "Unused variable in line 3".to_string(),
        "test.z1c".to_string(),
    );

    // All errors should be collected
    assert_eq!(collector.error_count(), 3);
    assert_eq!(collector.warning_count(), 1);
    assert_eq!(collector.diagnostics().len(), 4);
}

#[test]
fn test_warning_suppression_with_warn_level_none() {
    let config = DiagnosticConfig {
        use_colors: false,
        warn_level: WarnLevel::None,
        warn_as_error: false,
        max_errors: 50,
        json_output: false,
    };

    // When warn_level is None, warnings could be filtered
    // This test verifies the configuration exists
    assert_eq!(config.warn_level, WarnLevel::None);
}

#[test]
fn test_json_output_mode() {
    let config = DiagnosticConfig {
        use_colors: false,
        warn_level: WarnLevel::Default,
        warn_as_error: false,
        max_errors: 50,
        json_output: true,
    };

    assert!(config.json_output);

    // Verify JSON serialization works
    let diag = Diagnostic::error("Test".to_string(), "test.z1c".to_string());
    let json = serde_json::to_string(&diag).unwrap();
    assert!(json.contains("error"));
    assert!(json.contains("Test"));
}

#[test]
fn test_max_errors_limit() {
    let config = DiagnosticConfig {
        use_colors: false,
        warn_level: WarnLevel::Default,
        warn_as_error: false,
        max_errors: 10,
        json_output: false,
    };

    assert_eq!(config.max_errors, 10);

    // Verify we can collect up to max_errors
    let mut collector = DiagnosticCollector::new();
    for i in 0..15 {
        collector.add_error(format!("Error {}", i), "test.z1c".to_string());
    }

    // All errors collected (enforcement would happen at display time)
    assert_eq!(collector.error_count(), 15);
}
