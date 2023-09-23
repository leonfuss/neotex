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

extern crate tracing;

fn main() {
    // setup tracing
    tracing_subscriber::fmt()
        .pretty()
        .with_thread_names(true)
        .with_max_level(tracing::Level::ERROR)
        // sets this to be the default, global collector for this application.
        .init();
}
