use crate::{preparse::LexedStr, syntax::SyntaxKind, SyntaxKind::*};

#[test]
fn single_command() {
    let input = r#"\def:Nn_asdf:a {}"#;
    let expected = vec![Command, Whitespace, OpenBrace, CloseBrace];
    check(input, &expected, 0, 0);
}

#[test]
fn command_trainling_underscore() {
    let input = r#"\def:Nn_asdf:a_"#;
    let expected = vec![Error];
    check(input, &expected, 1, 0);
}

#[test]
fn command_trainling_underscore_whitespace() {
    let input = r#"\def:Nn_asdf:a_ "#;
    let expected = vec![Error, Whitespace];
    check(input, &expected, 1, 0);
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

fn check(input: &str, expected: &[SyntaxKind], num_errors: u8, num_definitions: u8) {
    let lexed = LexedStr::new(input);
    let expected = expected.to_vec();
    let found: Vec<SyntaxKind> = lexed.syntax_tokens().copied().collect();

    let found_num_errors = lexed.errors().count();
    let found_num_definitions = lexed.definitions().count();

    assert_eq!(expected, found);
    assert_eq!(num_errors as usize, found_num_errors as usize);
    assert_eq!(num_definitions as usize, found_num_definitions);
}
