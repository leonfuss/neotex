use std::fmt::Debug;

use crate::token::{
    tokenize,
    TokenKind::{self, *},
};

#[test]
fn word() {
    let input: &str = "asdjfqwertzuiopoonmnyxc";
    let expected = vec![AWord];
    check(input, &expected);
}

#[test]
fn words() {
    let input = "asdfäüihnß  asdf";
    let expected = vec![AWord, Word, AWord, Word, Whitespace, AWord];
    check(input, &expected);
}

#[test]
fn words_whitespace() {
    let input = "as         df jhljl";
    let expected = vec![AWord, Whitespace, AWord, Whitespace, AWord];
    check(input, &expected);
}

#[test]
fn words_newline_single_char() {
    let input: &str = "asdf\njhljl\n";
    let expected = vec![AWord, Newline, AWord, Newline];
    check(input, &expected);
}

#[test]
fn words_newline_multi_char() {
    let input = "asdf\r\njhljl\n";
    let expected = vec![AWord, Newline, AWord, Newline];
    check(input, &expected);
}
#[test]
fn multi_newline() {
    let input = "\n\r\n\n\r\n\n\n";
    let expected = vec![Newline, Newline, Newline, Newline, Newline, Newline];
    check(input, &expected);
}

#[test]
fn comment() {
    let input = "asdf 98    % asdfl#ü124<ääp\n % kllk \r\n%";
    let expected = vec![
        AWord, Whitespace, Number, Whitespace, Comment, Whitespace, Comment, Comment,
    ];
    check(input, &expected);
}

#[test]
fn command_1() {
    let input = "\\0\\+ \\asdf";
    let expected = vec![
        CommandIdent,
        Number,
        CommandIdent,
        Plus,
        Whitespace,
        CommandIdent,
        AWord,
    ];
    check(input, &expected);
}

#[test]
fn command_2() {
    let input = "\\\\\\0 \\+ \\asdf";
    let expected = vec![
        CommandIdent,
        CommandIdent,
        CommandIdent,
        Number,
        Whitespace,
        CommandIdent,
        Plus,
        Whitespace,
        CommandIdent,
        AWord,
    ];
    check(input, &expected);
}

fn check(input: &str, expected: &[TokenKind]) {
    let lexed: Vec<TokenKind> = tokenize(input).map(|it| it.kind).collect();

    assert_eq_token(lexed.into_iter(), expected);
}

// fn check_err(input: &str, expected: &[TokenKind], err: &[TokenKind]) {
//     let lexed = tokenize(input).map(|it| it.kind).into_iter();
//     assert_eq_token(lexed, expected);
// }

fn assert_eq_token<T>(token: impl Iterator<Item = T>, expected: &[T])
where
    T: Debug + Clone + PartialEq,
{
    let token: Vec<T> = token.collect();
    assert_eq!(token, expected);
}
