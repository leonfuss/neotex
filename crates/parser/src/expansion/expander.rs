use super::{resexp_parser::ResExpParser, store::ExpansionStore};
use crate::{preparse::LexedStr, SyntaxKind};

// note: functions must not be expanded. Functions are signified with opening param after the name
// input: all commands and environments that can be expanded. For environments, we search for
// \begin{env_name} and \end{env_name}. For commands, we search for \cmd_name{...}
// After finding a command or enviroments we check if the name is contained in the store. If it is
// we expand it. If it is not we raise a warning, to be sure that the later analysis is equal to
// the initial one. If an error is raised, we do not expand and return the error.
//
// Recursion: If we find recursion, in the expansion, we stop after the recursion limit.
//
// \expandafter: We skip ahead to the next token and try to expand it. If that is not possible, we
// simply return the token.
pub(crate) fn expand(lexed: &mut LexedStr, store: &ExpansionStore) {}

fn expand_def() {
    // TODO: implement pattern matching for \def
    expand_command();
}

fn expand_command() {}

fn expand_environment() {}

// TODO: replace with exander
fn expandafter(mut r: ResExpParser) {
    let m = r.mark();
    // skip \expandafter
    r.advance();

    if r.peek() == SyntaxKind::Command {
        expand_command();
    }

    // skip back to m
    // r.restore(m);
}
