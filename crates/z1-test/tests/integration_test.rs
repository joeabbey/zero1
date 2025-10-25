use z1_test::{parse_test_file, TestRunner};

#[test]
fn parse_simple_fixture() {
    let source =
        std::fs::read_to_string("../../fixtures/tests/simple.z1t").expect("Failed to read fixture");

    let result = parse_test_file(&source);
    assert!(
        result.is_ok(),
        "Failed to parse simple.z1t: {:?}",
        result.err()
    );

    let file = result.unwrap();
    assert_eq!(file.config.timeout_ms, Some(3000));
    assert_eq!(file.specs.len(), 3);
}

#[test]
fn parse_property_fixture() {
    let source = std::fs::read_to_string("../../fixtures/tests/property.z1t")
        .expect("Failed to read fixture");

    let result = parse_test_file(&source);
    assert!(
        result.is_ok(),
        "Failed to parse property.z1t: {:?}",
        result.err()
    );

    let file = result.unwrap();
    assert_eq!(file.props.len(), 2);
}

#[test]
fn parse_mixed_fixture() {
    let source =
        std::fs::read_to_string("../../fixtures/tests/mixed.z1t").expect("Failed to read fixture");

    let result = parse_test_file(&source);
    assert!(
        result.is_ok(),
        "Failed to parse mixed.z1t: {:?}",
        result.err()
    );

    let file = result.unwrap();
    assert_eq!(file.fixtures.len(), 1);
    assert_eq!(file.specs.len(), 3);
    assert_eq!(file.props.len(), 1);
}

#[test]
fn run_simple_fixture() {
    let source =
        std::fs::read_to_string("../../fixtures/tests/simple.z1t").expect("Failed to read fixture");

    let file = parse_test_file(&source).unwrap();
    let mut runner = TestRunner::default();
    let results = runner.run_file(&file);

    assert_eq!(results.passed, 3);
    assert_eq!(results.failed, 0);
    assert_eq!(results.skipped, 0);
}

#[test]
fn run_property_fixture() {
    let source = std::fs::read_to_string("../../fixtures/tests/property.z1t")
        .expect("Failed to read fixture");

    let file = parse_test_file(&source).unwrap();
    let mut runner = TestRunner::default();
    let results = runner.run_file(&file);

    // Both property tests should pass
    assert_eq!(results.passed, 2);
    assert_eq!(results.failed, 0);
}

#[test]
fn run_mixed_fixture_with_skip() {
    let source =
        std::fs::read_to_string("../../fixtures/tests/mixed.z1t").expect("Failed to read fixture");

    let file = parse_test_file(&source).unwrap();
    let mut runner = TestRunner::default();
    let results = runner.run_file(&file);

    // 2 specs + 1 prop pass, 1 spec skipped
    assert_eq!(results.passed, 3);
    assert_eq!(results.skipped, 1);
}
