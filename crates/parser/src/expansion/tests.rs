use rustc_hash::FxHashMap;

use super::{
    resolve,
    store::{ExpansionArgs, ExpansionStoreItem},
};
use crate::preparse::LexedStr;

#[test]
fn simple() {
    let src = "\\newcommand{\\name}{as#1dfasdf}";
    let args = ExpansionArgs {
        arg_count: 0,
        optional: Vec::default(),
        optional_named: FxHashMap::default(),
    };
    let mut req_expansion = FxHashMap::default();
    req_expansion.insert(1, vec![6]);

    let expected = ExpansionStoreItem {
        body: 5..8,
        second_body: None,
        name: "\\name",
        kind: crate::preparse::DefinitionKind::Macro,
        opt_expansion: FxHashMap::default(),
        req_expansion,
        args,
    };

    check(src, expected)
}

#[test]
fn arg_count_without_bracket() {
    let src = "\\newcommand{\\name}[3]{as#1dfasdf#3}";
    let args = ExpansionArgs {
        arg_count: 3,
        optional: Vec::default(),
        optional_named: FxHashMap::default(),
    };
    let mut req_expansion = FxHashMap::default();
    req_expansion.insert(1, vec![9]);
    req_expansion.insert(3, vec![11]);

    let expected = ExpansionStoreItem {
        body: 8..12,
        second_body: None,
        name: "\\name",
        kind: crate::preparse::DefinitionKind::Macro,
        opt_expansion: FxHashMap::default(),
        req_expansion,
        args,
    };

    check(src, expected)
}

#[test]
fn optional_arg() {
    let src = "\\newcommand{\\name}[1][Leon Fuss]{as#1dfasdf#3}";
    let args = ExpansionArgs {
        arg_count: 1,
        optional: vec![8..11],
        optional_named: FxHashMap::default(),
    };
    let mut req_expansion = FxHashMap::default();
    req_expansion.insert(1, vec![14]);
    req_expansion.insert(3, vec![16]);

    let expected = ExpansionStoreItem {
        body: 13..17,
        second_body: None,
        name: "\\name",
        kind: crate::preparse::DefinitionKind::Macro,
        opt_expansion: FxHashMap::default(),
        req_expansion,
        args,
    };

    check(src, expected)
}

#[test]
fn optional_named_arg() {
    let src = "\\newcommand{\\name}[1][name = Leon Fuss]{as#name dfasdf#3}";
    let mut optional_named = FxHashMap::default();
    optional_named.insert("name", 11..15);
    let args = ExpansionArgs {
        arg_count: 1,
        optional: Vec::default(),
        optional_named,
    };
    let mut req_expansion = FxHashMap::default();
    req_expansion.insert(3, vec![21]);
    let mut opt_expansion = FxHashMap::default();
    opt_expansion.insert("name", vec![18]);

    let expected = ExpansionStoreItem {
        body: 17..22,
        second_body: None,
        name: "\\name",
        kind: crate::preparse::DefinitionKind::Macro,
        opt_expansion,
        req_expansion,
        args,
    };

    check(src, expected)
}

#[test]
// test declaration of a new environment using \neuenvironment command
fn new_environment() {
    let src =
        "\\newenvironment{env_name}[1][Leon Fuss]{as#1dfasdf#3} {asdf#2}";
    let args = ExpansionArgs {
        arg_count: 1,
        optional: vec![10..13],
        optional_named: FxHashMap::default(),
    };
    let mut req_expansion = FxHashMap::default();
    req_expansion.insert(1, vec![16]);
    req_expansion.insert(3, vec![18]);
    req_expansion.insert(2, vec![23]);
    let expected = ExpansionStoreItem {
        body: 15..19,
        second_body: Some(22..24),
        name: "env_name",
        kind: crate::preparse::DefinitionKind::Environment,
        opt_expansion: FxHashMap::default(),
        req_expansion,
        args,
    };
    check(src, expected)
}

fn check(src: &str, expected: ExpansionStoreItem) {
    let lexed = LexedStr::new(src);
    dbg!(&lexed);
    let res = resolve(&lexed);
    dbg!(&res);
    let found = res.0.get(expected.name).expect("given key not found");
    if let Some(range) = found.args.optional.first() {
        let begin = range.start;
        let iter = lexed.iter_from(begin);
        iter.take(range.end - begin).for_each(|it| println!("{:?}", it));
    };

    assert_eq!(*found, expected);
}
