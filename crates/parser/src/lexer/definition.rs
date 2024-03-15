use std::ops::Deref;

use unicode_ident::{is_xid_continue, is_xid_start};

use crate::lexer::{
    infra::{consume, consume_str, reconsume, reset, LexerDelegate, LexerNext},
    tables::{COMPOSITE_SYMBOL_TABLE, SYMBOL_TABLE, UNIT_TABLE},
    token::LexToken,
};

use super::infra::TokenizerItemDelegate;

#[derive(Debug, Clone)]
pub(crate) enum LexerState {
    // unit is true if a number occured before the whitespace. This is needed to allow
    // for units to be parsed after a number even with a space or a single newline in between.
    Top { unit: bool },
    Word,
    AWord,
    UWord,
    Symbol,
    CommandNameBegin,
    CommandNameContinueBegin,
    CommandNameContinue,
    VariableName,
    VariableNameStart,
    VariableNameContinue,
    Comment,
    Number,
    Float,
    FloatWithExponent,
    Unit,
    Whitespace { unit: bool, from: FromSeperator, len: usize },
    Newline { unit: bool },
    Break,
    MacroParameter,
    MacroParameterContinue,
    UnicodeEscape,
    UnicodeEscapeValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FromSeperator {
    None,
    Newline,
    Break,
}

pub(crate) type TokenizerItem<'table> = TokenizerItemDelegate<'table, LexerState>;

impl LexerDelegate for LexerState {
    type Token = LexToken;

    fn top() -> LexerState {
        LexerState::Top { unit: false }
    }

    fn next<'src>(&self, c: Option<char>, rest: &'src str) -> LexerNext<Self> {
        use LexerState::*;

        match self {
            &Top { unit } => match c {
                None => LexerNext::EOF,
                Some(c) => match c {
                    // control sequences
                    c if is_whitespace(c) => {
                        LexerNext::begin(Whitespace { unit, from: FromSeperator::None, len: 0 })
                    }
                    c if is_break(c) => LexerNext::begin(Break),
                    c if is_multichar_newline(c, rest) => {
                        consume_str("\r\n").and_transition(Newline { unit })
                    }
                    c if is_newline(c, rest) => consume(c).and_transition(Newline { unit }),

                    // numbers
                    '0'..='9' => LexerNext::begin(Number),
                    '.' if is_continue_numeric(rest) => LexerNext::begin(Float),
                    // all units are ascii
                    c if (unit && c.is_ascii()) => reconsume().and_transition(Unit),

                    // could be a macro, environment, variable or unicode escape
                    '\\' => consume('\\')
                        .and_emit(LexToken::CommandIdent)
                        .and_transition(CommandNameBegin),

                    '#' => LexerNext::begin(MacroParameter),

                    '%' => LexerNext::begin(Comment),
                    c if !c.is_alphabetic() => LexerNext::begin(Symbol),
                    _ => LexerNext::begin(Word),
                },
            },

            &Whitespace { unit, from, len } => match c {
                None => reconsume().and_emit(LexToken::Whitespace).and_transition(top()),
                Some(c) if is_newline(c, rest) && from != FromSeperator::None => {
                    reconsume().and_transition(Break)
                }
                Some(c) if is_whitespace(c) => {
                    consume(c).and_transition(Whitespace { unit, from, len: len + c.len_utf8() })
                }
                Some(_) if from == FromSeperator::Break => {
                    reset(len).and_emit(LexToken::Break).and_transition(top())
                }
                Some(_) if from == FromSeperator::Newline => {
                    reset(len).and_emit(LexToken::Newline).and_transition(top())
                }
                Some(_) => reconsume().and_emit(LexToken::Whitespace).and_transition(Top { unit }),
            },

            Break => match c {
                None => reconsume().and_emit(LexToken::Break).and_transition(top()),
                Some(c) if is_whitespace(c) => reconsume().and_transition(Whitespace {
                    unit: false,
                    from: FromSeperator::Break,
                    len: 0,
                }),
                Some(c) if is_multichar_newline(c, rest) => consume_str("\r\n").and_remain(),
                Some(c) if is_newline(c, rest) => consume(c).and_remain(),
                Some(_) => reconsume().and_emit(LexToken::Break).and_transition(top()),
            },

            &Newline { unit } => match c {
                None => reconsume().and_emit(LexToken::Newline).and_transition(top()),
                Some(c) if is_whitespace(c) => reconsume().and_transition(Whitespace {
                    unit,
                    from: FromSeperator::Newline,
                    len: 0,
                }),
                Some(c) if is_multichar_newline(c, rest) => {
                    consume_str("\r\n").and_transition(Break)
                }
                Some(c) if is_newline(c, rest) => consume(c).and_transition(Break),
                Some(_) => reconsume().and_emit(LexToken::Newline).and_transition(Top { unit }),
            },

            Number => match c {
                None => reconsume().and_emit(LexToken::Integer).and_transition(top()),
                Some(c @ '0'..='9') => consume(c).and_remain(),
                Some('_') if is_continue_numeric(rest) => consume('_').and_remain(),
                Some('.') if is_continue_numeric(rest) => consume('.').and_transition(Float),
                Some('e') if is_continue_minus_numeric(rest) => {
                    consume_str("e-").and_transition(FloatWithExponent)
                }
                Some('e') if is_continue_numeric(rest) => {
                    consume('e').and_transition(FloatWithExponent)
                }
                Some(_) => reconsume().and_emit(LexToken::Integer).and_transition(top()),
            },

            Float => match c {
                None => reconsume().and_emit(LexToken::Float).and_transition(top()),
                Some(c @ '0'..='9') => consume(c).and_remain(),
                Some('_') if is_continue_numeric(rest) => consume('_').and_remain(),
                Some('e') if is_continue_minus_numeric(rest) => {
                    consume_str("e-").and_transition(FloatWithExponent)
                }
                Some('e') if is_continue_numeric(rest) => {
                    consume('e').and_transition(FloatWithExponent)
                }
                Some(_) => reconsume().and_emit(LexToken::Float).and_transition(top()),
            },

            FloatWithExponent => match c {
                None => reconsume().and_emit(LexToken::Float).and_transition(top()),
                Some(c @ '0'..='9') => consume(c).and_remain(),
                Some('_') if is_continue_numeric(rest) => consume('_').and_remain(),
                Some(_) => reconsume().and_emit(LexToken::Float).and_transition(top()),
            },

            Unit => match c {
                None => reconsume().and_discard().and_transition(top()),
                // 'b'..='s' is the bound of the unit table
                Some(first @ 'b'..='s') => {
                    let mut chars = rest.chars().take(2);

                    let unit = chars.next().map_or(false, |second| {
                        UNIT_TABLE.iter().any(|(f, s)| f == &first && s == &second)
                    });
                    let boundry = chars.next().map_or(true, |c| !c.is_alphabetic());

                    if unit && boundry {
                        // all units have the same utf-8 length
                        consume_str("mm").and_transition(top())
                    } else {
                        reconsume().and_transition(Word)
                    }
                }
                Some(_) => reconsume().and_transition(Word),
            },

            CommandNameBegin => match c {
                None => reconsume().and_discard().and_transition(top()),
                Some(c) if is_whitespace(c) => reconsume().and_discard().and_transition(top()),
                Some(c) if is_newline(c, rest) => reconsume().and_discard().and_transition(top()),
                Some('u') => consume('u').and_transition(UnicodeEscape),
                Some('@') => {
                    consume('@').and_emit(LexToken::VariableIdent).and_transition(VariableName)
                }
                Some(':') if rest.starts_with(':') => consume_str("::")
                    .and_emit(LexToken::PathSeparator)
                    .and_transition(CommandNameContinueBegin),
                Some(c) if is_xid_start(c) => consume(c).and_transition(CommandNameContinue),
                Some(c) if c.is_ascii() => {
                    consume(c).and_emit(LexToken::Command).and_transition(top())
                }
                Some(_) => reconsume().and_transition(Word),
            },

            // only called after a `::` has been consumed
            CommandNameContinueBegin => match c {
                None => reconsume().and_discard().and_transition(top()),
                // abort if an '_' follows
                Some('_') => reconsume().and_transition(top()),
                Some(c) if is_xid_start(c) => consume(c).and_transition(CommandNameContinue),
                Some(_) => reconsume().and_transition(Word),
            },

            CommandNameContinue => match c {
                None => reconsume().and_emit(LexToken::Command).and_transition(top()),
                Some(':') if rest.starts_with(':') => consume_str("::")
                    .and_emit(LexToken::PathSeparator)
                    .and_transition(CommandNameContinueBegin),
                Some(c) if is_xid_continue(c) => consume(c).and_remain(),
                Some(_) => reconsume().and_emit(LexToken::Command).and_transition(top()),
            },

            // only called after '\@' has been consumed
            VariableName => match c {
                None => reconsume().and_discard().and_transition(top()),
                Some(':') if rest.starts_with(':') => consume_str("::")
                    .and_emit(LexToken::PathSeparator)
                    .and_transition(VariableNameStart),
                Some(c) if is_xid_start(c) => consume(c).and_transition(VariableNameContinue),
                Some(_) => reconsume().and_transition(Word),
            },

            VariableNameStart => match c {
                None => reconsume().and_discard().and_transition(top()),
                Some(c) if is_xid_start(c) => consume(c).and_transition(VariableNameContinue),
                Some(_) => reconsume().and_transition(Word),
            },

            VariableNameContinue => match c {
                None => reconsume().and_emit(LexToken::Variable).and_transition(top()),
                Some(':') if rest.starts_with(':') => consume_str("::")
                    .and_emit(LexToken::PathSeparator)
                    .and_transition(VariableNameStart),
                Some(c) if is_xid_continue(c) => consume(c).and_remain(),
                Some(_) => reconsume().and_emit(LexToken::Variable).and_transition(top()),
            },

            UnicodeEscape => match c {
                None => reconsume().and_emit(LexToken::UnicodeEscape).and_transition(top()),
                Some('{') => consume('{').and_transition(UnicodeEscapeValue),
                Some(c) if is_whitespace(c) || is_newline(c, rest) => {
                    reconsume().and_emit(LexToken::UnicodeEscape).and_transition(top())
                }
                Some(_) => reconsume().and_transition(CommandNameContinue),
            },

            UnicodeEscapeValue => match c {
                None => reconsume().and_emit(LexToken::UnicodeEscape).and_transition(top()),
                Some('}') => consume('}').and_emit(LexToken::UnicodeEscape).and_transition(top()),
                Some(c) => consume(c).and_remain(),
            },

            MacroParameter => match c {
                None => reconsume().and_emit(LexToken::NumSign).and_transition(top()),
                Some(c @ '0'..='9') => consume(c).and_transition(MacroParameterContinue),
                Some(c) if is_xid_start(c) => consume(c).and_transition(MacroParameterContinue),
                Some(_) => reconsume().and_emit(LexToken::NumSign).and_transition(top()),
            },

            MacroParameterContinue => match c {
                None => reconsume().and_emit(LexToken::MacroParameter).and_transition(top()),
                Some('_') => reconsume().and_emit(LexToken::MacroParameter).and_transition(top()),
                Some(c) if is_xid_continue(c) => consume(c).and_remain(),
                Some(_) => reconsume().and_emit(LexToken::MacroParameter).and_transition(top()),
            },

            Comment => match c {
                None => reconsume().and_emit(LexToken::Comment).and_transition(top()),
                Some(c) if is_newline(c, rest) => {
                    reconsume().and_emit(LexToken::Comment).and_transition(top())
                }
                Some(c) => consume(c).and_remain(),
            },

            Symbol => match c {
                None => reconsume().and_discard().and_transition(top()),
                Some(first) => {
                    if let Some(second) = rest.chars().next() {
                        if let Some((.., token)) = COMPOSITE_SYMBOL_TABLE
                            .iter()
                            .find(|&(f, s, _)| f == &first && s == &second)
                        {
                            return consume_str("$$").and_emit(*token).and_transition(top());
                        }
                    }

                    let token = SYMBOL_TABLE
                        .iter()
                        .find(|(c, _)| c == &first)
                        .map_or(LexToken::Symbol, |(_, token)| *token);

                    consume(first).and_emit(token).and_transition(top())
                }
            },

            Word => match c {
                None => reconsume().and_transition(top()),
                Some(c) if !c.is_ascii_alphabetic() => reconsume().and_transition(AWord),
                Some(c) => reconsume().and_transition(UWord),
            },

            AWord => match c {
                None => reconsume().and_emit(LexToken::AWord).and_transition(top()),
                Some(c) if !c.is_ascii_alphabetic() => {
                    reconsume().and_emit(LexToken::AWord).and_transition(top())
                }
                Some(c) => consume(c).and_remain(),
            },

            UWord => match c {
                None => reconsume().and_emit(LexToken::UWord).and_transition(top()),
                Some(c) if !c.is_alphabetic() => {
                    reconsume().and_emit(LexToken::UWord).and_transition(top())
                }
                Some(c) => consume(c).and_remain(),
            },
        }
    }
}

fn top() -> LexerState {
    LexerState::top()
}

fn is_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'   // \t
           | '\u{0020}' // space
           | '\u{00A0}' // no-break space
           | '\u{1680}' // ogham space mark
           | '\u{202F}' // narrow no-break space
           | '\u{3000}' // ideographic space

           // Bidi markers
           | '\u{200E}' // LEFT-TO-RIGHT MARK
           | '\u{200F}' // RIGHT-TO-LEFT MARK
    )
}

fn is_newline(c: char, rest: &str) -> bool {
    if c == '\r' && rest.starts_with('\n') {
        return true;
    }
    matches!(
        c,
        '\u{000A}' // \n
           | '\u{000B}' // vertical tab
           | '\u{000C}' // form feed
           | '\u{000D}' // \r
           | '\u{0085}' // NEXT LINE from latin1

           // Dedicated whitespace characters from Unicode
           | '\u{2028}' // LINE SEPARATOR
           | '\u{2029}' // PARAGRAPH SEPARATOR
    )
}

fn is_multichar_newline(c: char, rest: &str) -> bool {
    c == '\r' && rest.starts_with('\n')
}

fn is_break(c: char) -> bool {
    matches!(c, '\u{2029}') // PARAGRAPH SEPARATOR
}

fn is_continue_numeric(rest: &str) -> bool {
    rest.starts_with(|c: char| matches!(c, '0'..='9'))
}

fn is_continue_minus_numeric(rest: &str) -> bool {
    let mut rest = rest.chars();
    rest.next().map_or(false, |c| c == '-') && rest.next().map_or(false, |c| matches!(c, '0'..='9'))
}
