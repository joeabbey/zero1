use z1_fmt::{format_module, FmtOptions, Mode};
use z1_hash::module_hashes;
use z1_parse::parse_module;

fn read_fixture(path: &str) -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(path);
    std::fs::read_to_string(root).expect("fixture not found")
}

#[test]
fn formats_compact_fixture() {
    let source = read_fixture("fixtures/cells/http_server.z1c");
    let module = parse_module(&source).expect("parse");
    let formatted = format_module(&module, Mode::Compact, &FmtOptions::default()).expect("fmt");
    let expected = read_fixture("fixtures/fmt/http_server.compact.z1c");
    assert_eq!(formatted, expected);
}

#[test]
fn formats_relaxed_fixture() {
    let source = read_fixture("fixtures/cells/http_server.z1c");
    let module = parse_module(&source).expect("parse");
    let formatted = format_module(&module, Mode::Relaxed, &FmtOptions::default()).expect("fmt");
    let expected = read_fixture("fixtures/fmt/http_server.relaxed.z1r");
    assert_eq!(formatted, expected);
}

#[test]
fn round_trip_preserves_semantics() {
    let source = read_fixture("fixtures/cells/http_server.z1c");
    let module = parse_module(&source).expect("parse");
    let original_hash = module_hashes(&module).semantic;

    let compact = format_module(&module, Mode::Compact, &FmtOptions::default()).expect("fmt");
    let reparsed_compact = parse_module(&compact).expect("parse compact");
    let relaxed = format_module(&reparsed_compact, Mode::Relaxed, &FmtOptions::default())
        .expect("fmt relaxed");
    let reparsed_relaxed = parse_module(&relaxed).expect("parse relaxed");
    let final_hash = module_hashes(&reparsed_relaxed).semantic;

    assert_eq!(original_hash, final_hash);
}

#[test]
fn formats_statements_fixture() {
    let source = read_fixture("fixtures/fmt/statements.compact.z1c");
    let module = parse_module(&source).expect("parse");
    let relaxed = format_module(&module, Mode::Relaxed, &FmtOptions::default()).expect("fmt");
    let expected_relaxed = read_fixture("fixtures/fmt/statements.relaxed.z1r");
    assert_eq!(relaxed, expected_relaxed);

    let reparsed = parse_module(&relaxed).expect("parse relaxed");
    let compact =
        format_module(&reparsed, Mode::Compact, &FmtOptions::default()).expect("fmt compact");
    let expected_compact = read_fixture("fixtures/fmt/statements.compact.z1c");
    assert_eq!(compact, expected_compact);
}
