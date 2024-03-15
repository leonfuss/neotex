use crate::lexer::LexToken;

/// A set of tokens to easily match against membership. Uses a bitset to store the tokens.

pub(crate) struct TokenSet(u128);

impl TokenSet {
    const EMPTY: TokenSet = TokenSet(0);

    const fn mask(token: LexToken) -> u128 {
        1 << (token as u128)
    }

    pub const fn new() -> TokenSet {
        TokenSet::EMPTY
    }

    pub const fn from_slice(tokens: &[LexToken]) -> TokenSet {
        let mut set = TokenSet::EMPTY;
        let mut i = 0;
        while i < tokens.len() {
            let token = tokens[i];
            if !set.contains(token) {
                set = set.insert(token);
            }
            i += 1;
        }
        set
    }

    pub const fn insert(self, token: LexToken) -> TokenSet {
        TokenSet(self.0 | TokenSet::mask(token))
    }

    pub const fn contains(&self, token: LexToken) -> bool {
        self.0 & TokenSet::mask(token) != 0
    }
}
