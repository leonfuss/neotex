use std::fmt::Debug;

use crate::{
    lexed_str::{LexErrorKind, LexedStr},
    syntax::SyntaxKind::{self, *},
};

#[test]
fn word() {
    let input: &str = "asdjfqwertzuiopoonmnyxc";
    let expected = vec![ASCIIWORD, EOF];
    check(input, &expected);
}

#[test]
fn words() {
    let input = "asdfäüihnß";
    let expected = vec![ASCIIWORD, WORD, ASCIIWORD, WORD, EOF];
    check(input, &expected);
}

#[test]
fn words_whitespace() {
    let input = "asdf jhljl";
    let expected = vec![ASCIIWORD, WHITESPACE, ASCIIWORD, EOF];
    check(input, &expected);
}

#[test]
fn words_newline_single_char() {
    let input: &str = "asdf\njhljl\n";
    let expected = vec![ASCIIWORD, NEWLINE, ASCIIWORD, NEWLINE, EOF];
    check(input, &expected);
}

#[test]
fn words_newline_multi_char() {
    let input = "asdf\r\njhljl\n";
    let expected = vec![ASCIIWORD, NEWLINE, ASCIIWORD, NEWLINE, EOF];
    check(input, &expected);
}
#[test]
fn multi_newline() {
    let input = "\n\r\n\n\r\n\n\n";
    let expected = vec![NEWLINE, EOF];
    check(input, &expected);
}

#[test]
fn comment() {
    let input = "asdf 98    % asdfl#ü124<ääp\n % kllk";
    let expected = vec![
        ASCIIWORD, WHITESPACE, NUMBER, NUMBER, WHITESPACE, COMMENT, WHITESPACE, COMMENT, EOF,
    ];
    check(input, &expected);
}

#[test]
fn command() {
    let input = "\\\\\\0 \\+ \\asdf";
    let expected = vec![
        COMMAND, COMMAND, WHITESPACE, COMMAND, WHITESPACE, COMMAND, EOF,
    ];
    check(input, &expected);
}

#[test]
fn command_err() {
    let input = "\\ä \\";
    let expected = vec![UNKNOWN, WHITESPACE, UNKNOWN, EOF];
    let err = vec![
        LexErrorKind::InvalidCommandName,
        LexErrorKind::MissingCommandName,
    ];
    check_err(input, &expected, &err);
}

#[test]
fn unknown_char_err() {
    let input = "\u{1F600}";
    let expected = vec![UNKNOWN, EOF];
    let err = vec![LexErrorKind::UnknownChar];
    check_err(input, &expected, &err);
}

fn check(input: &str, expected: &[SyntaxKind]) {
    let lexed = LexedStr::new(input);

    assert_eq_token(lexed.token(), expected);
    assert_eq!(lexed.errors().count(), 0);
}

fn check_err(input: &str, expected: &[SyntaxKind], err: &[LexErrorKind]) {
    let lexed = LexedStr::new(input);
    assert_eq_token(lexed.token(), expected);
    assert_eq_token(lexed.errors().map(|it| it.1), err);
}

fn assert_eq_token<'a, T: 'a>(token: impl Iterator<Item = &'a T> + 'a, expected: &[T])
where
    T: Debug + Clone + PartialEq,
{
    let token: Vec<T> = token.into_iter().cloned().collect();
    assert_eq!(token, expected);
}
