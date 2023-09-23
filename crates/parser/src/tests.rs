use crate::{preparse::LexedStr, syntax::SyntaxKind, SyntaxKind::*};
#[test]
fn simple_command() {
    let input = r#"\a "#;
    let expected = vec![CommandOrFunction, Whitespace];
    check(input, &expected, 0, 0);
}

#[test]
fn simple_multichar_command() {
    let input = r#"\_:a "#;
    let expected = vec![Command, Whitespace];
    check(input, &expected, 0, 0);
}

#[test]
fn single_command() {
    let input = r#"\def:Nn_asdf:a {}"#;
    let expected = vec![Command, Whitespace, OpenBrace, CloseBrace];
    check(input, &expected, 0, 0);
}

#[test]
fn command_trailing_underscore() {
    let input = r#"\def:Nn_asdf:a_"#;
    let expected = vec![Command, Underscore];
    check(input, &expected, 0, 0);
}

#[test]
fn command_trailing_underscore_whitespace() {
    let input = r#"\def:Nn_asdf:a_ "#;
    let expected = vec![Command, Underscore, Whitespace];
    check(input, &expected, 0, 0);
}

#[test]
fn command_trainling_colon() {
    let input = r#"\def:Nn_asdf:a:"#;
    let expected = vec![Error];
    check(input, &expected, 1, 0);
}

#[test]
fn command_trainling_colon_whitespace() {
    let input = r#"\def:Nn_asdf:a_: "#;
    let expected = vec![Error, Whitespace];
    check(input, &expected, 1, 0);
}

#[test]
fn single_variable() {
    let input = r#"\@sdfAk_aaf4"#;
    let expected = vec![Variable];
    check(input, &expected, 0, 0);
}

#[test]
fn path_seperator_variable() {
    let input = r#"\::sd::fA::@k_aaf"#;
    let expected = vec![
        PathSeperator,
        Namespace,
        PathSeperator,
        Namespace,
        PathSeperator,
        Variable,
    ];
    check(input, &expected, 0, 0);
}
#[test]
fn path_seperator_function() {
    let input = r#"\::sd::fA::k_aaf"#;
    let expected = vec![
        PathSeperator,
        Namespace,
        PathSeperator,
        Namespace,
        PathSeperator,
        Function,
    ];
    check(input, &expected, 0, 0);
}

fn check(input: &str, expected: &[SyntaxKind], num_errors: u8, num_definitions: u8) {
    let lexed = LexedStr::new(input);
    println!("{:?}", lexed);
    let expected = expected.to_vec();
    let found: Vec<SyntaxKind> = lexed.syntax_tokens().copied().collect();

    let found_num_errors = lexed.errors().count();
    let found_num_definitions = lexed.definitions().count();

    assert_eq!(expected, found);
    assert_eq!(num_errors as usize, found_num_errors as usize);
    assert_eq!(num_definitions as usize, found_num_definitions);
}
