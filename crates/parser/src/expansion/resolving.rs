use super::errors::{ResolverErrorKind, ResolverResult};
use super::resexp_parser::OptionalKind;
use super::resolver::Resolver;
use crate::syntax::SyntaxKind::*;
use crate::utils::utils::Marker;

pub(super) fn def(r: &mut Resolver) -> ResolverResult {
    new_command(r)
}

pub(super) fn new_environment(r: &mut Resolver) -> ResolverResult {
    env_name(r)?;
    args(r)?;
    opt_args(r)?;
    body(r, true)?;
    body(r, false)?;
    Ok(())
}

pub(super) fn new_command(r: &mut Resolver) -> ResolverResult {
    cmd_name(r)?;
    args(r)?;
    opt_args(r)?;
    body(r, true)?;
    Ok(())
}

fn cmd_name(r: &mut Resolver) -> ResolverResult {
    let skip = r.open_brace(OptionalKind::OptionalBlockIdent)?;
    r.skip_trivia();
    if r.at(Command) {
        r.register_cmd_name()?;
        r.advance()?;
    }
    r.close_brace(skip)?;
    Ok(())
}

fn env_name(r: &mut Resolver) -> ResolverResult {
    let skip = r.open_brace(OptionalKind::Required)?;
    let m = r.mark();
    r.skip_trivia();
    match_name(r, m)?;
    r.register_env_name(m)?;
    r.close_brace(skip)?;
    Ok(())
}

fn args(r: &mut Resolver) -> ResolverResult {
    r.skip_trivia();

    if !(r.at(OpenBracket) || r.at_peek(Number)) {
        r.register_empty_arg_count();
        return Ok(());
    }

    let skip = r.open_bracket(OptionalKind::OptionalBlockIdent)?;

    r.skip_trivia();
    if r.at(Number) {
        r.register_arg_count()?;
        r.advance()?;
    }
    r.close_bracket(skip)?;
    Ok(())
}

fn opt_args(r: &mut Resolver) -> ResolverResult {
    let skip = r.open_bracket(OptionalKind::OptionalBlock)?;
    if skip {
        return Ok(());
    }
    let mut m = r.mark();
    loop {
        let _ = match_name(r, m);

        match r.peek_skip_trivia() {
            CloseBracket => {
                r.register_opt_arg(m)?;
                break;
            }
            Equal => {
                r.mark_opt_named_arg_name(m);
                opt_named_arg(r)?;
            }
            Comma => {
                r.register_opt_arg(m)?;
                r.advance()?;
                m = r.mark();
            }
            _ => {
                r.advance()?;
            }
        }

        if r.peek_skip_trivia() == CloseBracket {
            break;
        }
    }
    r.close_bracket(skip)?;
    Ok(())
}

fn match_name(r: &mut Resolver, begin: Marker) -> ResolverResult {
    match r.peek() {
        Underscore => r.advance()?,
        AWord => r.advance()?,
        t @ _ => {
            let name = r.text(*begin..r.index())?;
            return Err(ResolverErrorKind::InvalidName { name: name.to_string(), kind: t });
        }
    };

    loop {
        match r.peek() {
            Underscore if is_allowed_in_name(r) => r.advance()?,
            AWord | Number => r.advance()?,
            _ => break Ok(()),
        };
    }
}

fn opt_named_arg(r: &mut Resolver) -> ResolverResult {
    r.advance_to(Equal)?;
    debug_assert!(r.at(Equal));
    r.advance()?;
    let m = r.mark();
    loop {
        match r.peek() {
            CloseBracket => {
                r.register_opt_named_arg(m)?;
                break;
            }

            Comma => {
                r.register_opt_named_arg(m)?;
                r.advance()?;
                break;
            }
            t @ (OpenBracket | OpenBrace | CloseBrace) => {
                return Err(ResolverErrorKind::UnexpectedOpeningToken(t));
            }
            _ => r.advance()?,
        };
    }
    Ok(())
}

// match the body of a macro or environment definition and collect any simple or complex macro expansion tokens
fn body(r: &mut Resolver, first_body: bool) -> ResolverResult {
    let skip = r.open_brace(OptionalKind::Required)?;
    let m = r.mark();

    let mut brace_level = 0;
    loop {
        match r.peek() {
            t @ _ if brace_level < 0 => {
                return Err(ResolverErrorKind::UnexpectedToken { expected: OpenBrace, found: t })
            }
            CloseBrace if brace_level == 0 => break,
            OpenBrace => brace_level += 1,
            CloseBrace => brace_level -= 1,
            SimpleMacroExpansionToken => {
                r.register_smpl_macro_token()?;
            }
            ComplexMacroExpansionToken => {
                r.register_cmplx_macro_token()?;
            }
            _ => {}
        }
        r.advance()?;
    }

    if first_body {
        r.register_body(m);
    } else {
        r.register_second_body(m);
    }

    r.close_brace(skip)?;
    Ok(())
}

fn is_allowed_in_name(r: &Resolver) -> bool {
    matches!(r.peek_second(), AWord | Underscore | Number)
}
