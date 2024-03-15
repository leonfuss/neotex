#![doc(
    html_logo_url = "https://gist.githubusercontent.com/leonfuss/9247909d6237cb406439944fe22405a4/raw/02853019c2e0187bfb518dba1052ed5338c144c8/logo.svg"
)]
#![doc(
    html_favicon_url = "https://gist.githubusercontent.com/leonfuss/fa44f11267796b352edaa121675cd6f2/raw/ae237aa701a5d087c89af313299858f5e1c1144f/favicon.svg"
)]
#![deny(missing_docs)]

//! # Welcome to NeoTeX!
//!
//!
//!
//! # Crates
//! * [lexer](../lexer/index.html)
//! * [parser](../parser/index.html)

use std::{error::Error, path::PathBuf};

extern crate tracing;

// TODO: Remove and use better error handling
type Result<R> = std::result::Result<R, Box<dyn Error>>;

fn main() -> Result<()> {
    // setup tracing
    tracing_subscriber::fmt()
        .pretty()
        .with_thread_names(true)
        .with_max_level(tracing::Level::ERROR)
        // sets this to be the default, global collector for this application.
        .init();

    let args: Vec<String> = std::env::args().collect();
    if let Some(s) = args.get(1) {
        match s.as_str() {
            "tokens" if args.get(2).is_some() => token_stream(args.get(2).unwrap())?,

            s => println!("called unknown {s} or with false argument count"),
        }
    }
    Ok(())
}

fn token_stream(path: &str) -> Result<()> {
    let path = PathBuf::from(path);
    println!("reading {path:?}...",);
    let src = std::fs::read_to_string(path)?;

    println!("lexing input...");

    // println!("{lexed:?}");

    println!("resolving macors...");
    // let store = parser::expansion::resolve(&lexed);

    // println!("{store:?}");

    Ok(())
}
