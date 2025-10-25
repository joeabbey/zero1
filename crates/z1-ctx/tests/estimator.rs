use z1_ctx::{estimate_cell, estimate_cell_with_config, CtxError, EstimateConfig};
use z1_parse::parse_module;

#[test]
fn test_basic_estimation() {
    let source = r#"
m test:1.0
f hello()->Unit eff [pure] { ret Unit }
"#;

    let module = parse_module(source).unwrap();
    let estimate = estimate_cell(&module).unwrap();

    assert!(estimate.total_tokens > 0);
    assert_eq!(estimate.budget, None);
    assert_eq!(estimate.functions.len(), 1);
    assert_eq!(estimate.functions[0].name, "hello");
}

#[test]
fn test_estimation_with_budget_ok() {
    let source = r#"
m test:1.0 ctx=200
f hello()->Unit eff [pure] { ret Unit }
"#;

    let module = parse_module(source).unwrap();
    let estimate = estimate_cell(&module).unwrap();

    assert_eq!(estimate.budget, Some(200));
    assert!(estimate.total_tokens < 200, "Should be within budget");
}

#[test]
fn test_estimation_with_budget_exceeded() {
    // Create a minimal module with very small budget
    let source = r#"
m test:1.0 ctx=5
f hello()->Unit eff [pure] { ret Unit }
"#;

    let module = parse_module(source).unwrap();
    let result = estimate_cell(&module);

    match result {
        Err(CtxError::BudgetExceeded {
            actual,
            budget,
            suggestion,
            ..
        }) => {
            assert!(actual > budget);
            assert_eq!(budget, 5);
            assert!(!suggestion.is_empty(), "Should provide a suggestion");
        }
        _ => panic!("Expected BudgetExceeded error"),
    }
}

#[test]
fn test_no_enforce_budget() {
    let source = r#"
m test:1.0 ctx=5
f hello()->Unit eff [pure] { ret Unit }
"#;

    let module = parse_module(source).unwrap();
    let config = EstimateConfig {
        chars_per_token: z1_ctx::DEFAULT_CHARS_PER_TOKEN,
        enforce_budget: false,
    };

    let estimate = estimate_cell_with_config(&module, &config).unwrap();
    assert_eq!(estimate.budget, Some(5));
    assert!(estimate.total_tokens > 5);
}

#[test]
fn test_custom_chars_per_token() {
    let source = r#"
m test:1.0
f hello()->Unit eff [pure] { ret Unit }
"#;

    let module = parse_module(source).unwrap();

    // Test with default ratio
    let default_estimate = estimate_cell(&module).unwrap();

    // Test with custom ratio (higher = fewer tokens)
    let config = EstimateConfig {
        chars_per_token: 5.0,
        enforce_budget: false,
    };
    let custom_estimate = estimate_cell_with_config(&module, &config).unwrap();

    assert!(custom_estimate.total_tokens < default_estimate.total_tokens);
}

#[test]
fn test_multiple_functions() {
    let source = r#"
m test:1.0 ctx=500
f hello()->Unit eff [pure] { ret Unit }
f world()->Unit eff [pure] { ret Unit }
f foo()->Unit eff [pure] { ret Unit }
"#;

    let module = parse_module(source).unwrap();
    let estimate = estimate_cell(&module).unwrap();

    assert_eq!(estimate.functions.len(), 3);
    assert_eq!(estimate.functions[0].name, "hello");
    assert_eq!(estimate.functions[1].name, "world");
    assert_eq!(estimate.functions[2].name, "foo");

    for fn_est in &estimate.functions {
        assert!(fn_est.tokens > 0);
        assert!(fn_est.chars > 0);
    }
}

#[test]
fn test_http_server_fixture() {
    let source = r#"
m http.server:1.0 ctx=128 caps=[net]
#sym { handler: h, serve: sv }
u "std/http" as H only [listen, Req, Res]
t Health = { ok: Bool, msg: Str }

f h(q:H.Req)->H.Res eff [pure] {
  ret H.Res{ status:200, body:"ok" };
}

f sv(p:U16)->Unit eff [net] {
  H.listen(p, h);
}
"#;

    let module = parse_module(source).unwrap();
    let estimate = estimate_cell(&module).unwrap();

    assert_eq!(estimate.budget, Some(128));
    assert!(
        estimate.total_tokens <= 128,
        "http.server fixture should fit in budget. Got {} tokens",
        estimate.total_tokens
    );
    assert_eq!(estimate.functions.len(), 2);
}

#[test]
fn test_suggestion_for_single_function() {
    let source = r#"
m test:1.0 ctx=5
f bigfunction()->Unit eff [pure] { ret Unit }
"#;

    let module = parse_module(source).unwrap();
    let result = estimate_cell(&module);

    match result {
        Err(CtxError::BudgetExceeded { suggestion, .. }) => {
            assert!(suggestion.contains("bigfunction"));
            assert!(suggestion.contains("splitting"));
        }
        _ => panic!("Expected BudgetExceeded error"),
    }
}

#[test]
fn test_suggestion_for_multiple_functions() {
    let source = r#"
m test:1.0 ctx=5
f small()->Unit eff [pure] { ret Unit }
f large()->Unit eff [pure] { ret Unit }
"#;

    let module = parse_module(source).unwrap();
    let result = estimate_cell(&module);

    match result {
        Err(CtxError::BudgetExceeded { suggestion, .. }) => {
            assert!(suggestion.contains("separate cell") || suggestion.contains("moving"));
        }
        _ => panic!("Expected BudgetExceeded error"),
    }
}

#[test]
fn test_display_format() {
    let source = r#"
m test:1.0 ctx=200
f hello()->Unit eff [pure] { ret Unit }
"#;

    let module = parse_module(source).unwrap();
    let estimate = estimate_cell(&module).unwrap();

    let display = format!("{estimate}");
    assert!(display.contains("Cell Estimate"));
    assert!(display.contains("Total tokens"));
    assert!(display.contains("Budget"));
    assert!(display.contains("Usage"));
    assert!(display.contains("hello"));
}

#[test]
fn test_empty_module() {
    let source = "m empty:1.0";

    let module = parse_module(source).unwrap();
    let estimate = estimate_cell(&module).unwrap();

    assert!(estimate.total_tokens > 0, "Even empty module has tokens");
    assert_eq!(estimate.functions.len(), 0);
}

#[test]
fn test_module_with_imports_and_types() {
    let source = r#"
m test:1.0 ctx=300
u "std/core" as C only [Unit]
t MyType = { field: C.Unit }
f process()->C.Unit eff [pure] { ret C.Unit }
"#;

    let module = parse_module(source).unwrap();
    let estimate = estimate_cell(&module).unwrap();

    assert_eq!(estimate.budget, Some(300));
    assert!(estimate.total_tokens > 0);
    assert_eq!(estimate.functions.len(), 1);
}
