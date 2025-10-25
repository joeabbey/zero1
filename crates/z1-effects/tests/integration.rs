//! Integration tests for z1-effects using realistic module examples.

use z1_ast::{Block, FnDecl, Item, Module, ModulePath, Param, Span, TypeExpr};
use z1_effects::{check_module, EffectError};

fn make_module_with_caps(caps: Vec<&str>, functions: Vec<FnDecl>) -> Module {
    Module {
        path: ModulePath::from_parts(vec!["http".to_string(), "server".to_string()]),
        version: Some("1.0".to_string()),
        ctx_budget: Some(128),
        caps: caps.into_iter().map(String::from).collect(),
        items: functions.into_iter().map(Item::Fn).collect(),
        span: Span::new(0, 200),
    }
}

fn make_fn_with_effects(name: &str, effects: Vec<&str>, span: Span) -> FnDecl {
    FnDecl {
        name: name.to_string(),
        params: vec![Param {
            name: "arg".to_string(),
            ty: TypeExpr::Path(vec!["U16".to_string()]),
            span,
        }],
        ret: TypeExpr::Path(vec!["Unit".to_string()]),
        effects: effects.into_iter().map(String::from).collect(),
        body: Block {
            raw: "{ ret Unit; }".to_string(),
            statements: vec![],
            span,
        },
        span,
    }
}

#[test]
fn test_http_server_with_net_capability() {
    // Simulates the http_server.z1c fixture:
    // m http.server:1.0 ctx=128 caps=[net]
    // f h(q:H.Req)->H.Res eff [pure] { ... }
    // f sv(p:U16)->Unit eff [net] { ... }

    let functions = vec![
        make_fn_with_effects("handler", vec!["pure"], Span::new(50, 100)),
        make_fn_with_effects("serve", vec!["net"], Span::new(101, 150)),
    ];

    let module = make_module_with_caps(vec!["net"], functions);
    assert!(check_module(&module).is_ok());
}

#[test]
fn test_missing_net_capability() {
    let functions = vec![make_fn_with_effects(
        "serve",
        vec!["net"],
        Span::new(50, 100),
    )];

    let module = make_module_with_caps(vec![], functions);
    let result = check_module(&module);

    assert!(result.is_err());
    match result {
        Err(EffectError::MissingCapability {
            fn_name,
            effect,
            module,
            ..
        }) => {
            assert_eq!(fn_name, "serve");
            assert_eq!(effect, "net");
            assert_eq!(module, "http.server");
        }
        _ => panic!("Expected MissingCapability error"),
    }
}

#[test]
fn test_pure_function_in_module_without_caps() {
    let functions = vec![make_fn_with_effects(
        "pure_fn",
        vec!["pure"],
        Span::new(50, 100),
    )];

    let module = make_module_with_caps(vec![], functions);
    assert!(check_module(&module).is_ok());
}

#[test]
fn test_implicit_pure_function() {
    let functions = vec![make_fn_with_effects(
        "implicit_pure",
        vec![],
        Span::new(50, 100),
    )];

    let module = make_module_with_caps(vec![], functions);
    assert!(check_module(&module).is_ok());
}

#[test]
fn test_multiple_effects_with_all_caps() {
    let functions = vec![make_fn_with_effects(
        "complex_fn",
        vec!["net", "time", "fs"],
        Span::new(50, 100),
    )];

    let module = make_module_with_caps(vec!["net", "time", "fs"], functions);
    assert!(check_module(&module).is_ok());
}

#[test]
fn test_multiple_effects_missing_some_caps() {
    let functions = vec![make_fn_with_effects(
        "complex_fn",
        vec!["net", "time", "fs"],
        Span::new(50, 100),
    )];

    let module = make_module_with_caps(vec!["net", "time"], functions);
    let result = check_module(&module);

    assert!(result.is_err());
    match result {
        Err(EffectError::MissingCapability { effect, .. }) => {
            assert_eq!(effect, "fs");
        }
        _ => panic!("Expected MissingCapability error for 'fs'"),
    }
}

#[test]
fn test_unknown_effect_error() {
    let functions = vec![make_fn_with_effects(
        "bad_fn",
        vec!["unknown_effect"],
        Span::new(50, 100),
    )];

    let module = make_module_with_caps(vec![], functions);
    let result = check_module(&module);

    assert!(result.is_err());
    match result {
        Err(EffectError::UnknownEffect {
            fn_name, effect, ..
        }) => {
            assert_eq!(fn_name, "bad_fn");
            assert_eq!(effect, "unknown_effect");
        }
        _ => panic!("Expected UnknownEffect error"),
    }
}

#[test]
fn test_fine_grained_fs_capabilities() {
    // Test fs.ro capability
    let functions = vec![make_fn_with_effects(
        "read_fn",
        vec!["fs"],
        Span::new(50, 100),
    )];
    let module = make_module_with_caps(vec!["fs.ro"], functions);
    assert!(check_module(&module).is_ok());

    // Test fs.rw capability
    let functions = vec![make_fn_with_effects(
        "write_fn",
        vec!["fs"],
        Span::new(50, 100),
    )];
    let module = make_module_with_caps(vec!["fs.rw"], functions);
    assert!(check_module(&module).is_ok());
}

#[test]
fn test_all_effect_types() {
    let effects = vec![
        "pure", "net", "fs", "time", "crypto", "env", "async", "unsafe",
    ];

    for effect in &effects {
        let functions = vec![make_fn_with_effects(
            "test_fn",
            vec![effect],
            Span::new(50, 100),
        )];
        let module = make_module_with_caps(vec![effect], functions);

        if *effect == "pure" {
            // Pure doesn't need capability
            let module_no_caps = make_module_with_caps(
                vec![],
                vec![make_fn_with_effects(
                    "test_fn",
                    vec![effect],
                    Span::new(50, 100),
                )],
            );
            assert!(
                check_module(&module_no_caps).is_ok(),
                "Pure function should work without caps"
            );
        } else {
            assert!(
                check_module(&module).is_ok(),
                "Effect {effect} should be valid with matching capability"
            );
        }
    }
}

#[test]
fn test_case_insensitive_effect_matching() {
    let functions = vec![make_fn_with_effects("fn1", vec!["Net"], Span::new(50, 100))];
    let module = make_module_with_caps(vec!["net"], functions);
    assert!(check_module(&module).is_ok());

    let functions = vec![make_fn_with_effects("fn2", vec!["net"], Span::new(50, 100))];
    let module = make_module_with_caps(vec!["Net"], functions);
    assert!(check_module(&module).is_ok());

    let functions = vec![make_fn_with_effects("fn3", vec!["NET"], Span::new(50, 100))];
    let module = make_module_with_caps(vec!["NeT"], functions);
    assert!(check_module(&module).is_ok());
}

#[test]
fn test_mixed_pure_and_impure_functions() {
    let functions = vec![
        make_fn_with_effects("pure_helper", vec!["pure"], Span::new(20, 40)),
        make_fn_with_effects("net_handler", vec!["net"], Span::new(50, 80)),
        make_fn_with_effects("pure_formatter", vec![], Span::new(90, 120)),
        make_fn_with_effects("time_logger", vec!["time"], Span::new(130, 160)),
    ];

    let module = make_module_with_caps(vec!["net", "time"], functions);
    assert!(check_module(&module).is_ok());
}

#[test]
fn test_error_includes_span_information() {
    let functions = vec![make_fn_with_effects(
        "bad_fn",
        vec!["net"],
        Span::new(75, 125),
    )];

    let module = make_module_with_caps(vec![], functions);
    let result = check_module(&module);

    assert!(result.is_err());
    match result {
        Err(EffectError::MissingCapability { fn_span, .. }) => {
            assert_eq!(fn_span.start, 75);
            assert_eq!(fn_span.end, 125);
        }
        _ => panic!("Expected MissingCapability error with span"),
    }
}
