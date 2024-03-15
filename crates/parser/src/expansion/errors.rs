use std::ops::Range;

use thiserror::Error;

use crate::{preparse::DefinitionKind, utils::utils::Marker, SyntaxKind};

#[derive(Error, Debug)]
#[error("Error at {idx}: \n{source}")]
pub struct ResolverError {
    idx: usize,
    source: ResolverErrorKind,
}

pub(crate) type ResolverResult = Result<(), ResolverErrorKind>;

#[derive(Error, Debug)]
pub(crate) enum ResolverErrorKind {
    #[error("expected '{expected}', but found '{found}' at")]
    UnexpectedToken { expected: SyntaxKind, found: SyntaxKind },
    #[error("found {0} where no opening token is allowed")]
    UnexpectedOpeningToken(SyntaxKind),
    #[error("Invalid {kind} provided in {name}")]
    InvalidName { kind: SyntaxKind, name: String },
    #[error("Invalid argument count, {expected} where specified, but {found} found")]
    WrongArgumentCount { expected: usize, found: usize },
    #[error("Failed to retrieve text")]
    FailedToRetrieveText,
    #[error("Failed to retrieve text in range: {0:?}")]
    FailedToRetrieveTextRange(Range<usize>),
    #[error("failed to parse text at range: {0:?}")]
    FailedToParseText(Range<usize>),
    #[error("EOF reached while resoling definition")]
    EofReached,
}

impl ResolverErrorKind {
    pub(super) fn attach_marker(self, marker: Marker) -> ResolverError {
        ResolverError { idx: *marker, source: self }
    }
}
