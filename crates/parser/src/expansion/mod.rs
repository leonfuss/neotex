mod errors;
pub(crate) mod expanded;
mod expander;
mod expansion;
mod resexp_parser;
mod resolver;
mod resolving;
pub mod store;
pub(crate) use expander::expand;
pub(crate) use resolver::resolve;

#[cfg(test)]
mod tests;
