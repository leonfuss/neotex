/// The Kind of a Syntax Node
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u16)]
pub enum SyntaxKind {
    EOF,

    COMMAND,

    OBRACE,
    CBRACE,
    OPAREN,
    CPAREN,
    OBRACKET,
    CBRACKET,

    STAR,
    DOT,
    AT,
    SEMICOLON,
    COMMA,
    COLON,
    NUMSIGN,
    TILDE,
    DOLLAR,
    QUESTION,
    BANG,
    AMPERSAND,
    EQ,
    LT,
    GT,
    MINUS,
    PLUS,
    PIPE,
    CARRET,
    PERCENT,
    APOSTROPE,
    UNDERSCORE,
    SLASH,

    WHITESPACE,
    COMMENT, // lines starting with '%' including '\n' (!invariant)
    NEWLINE,

    WORD,
    ASCIIWORD,
    NUMBER,

    UNKNOWN,

    __LAST,
}

impl From<u16> for SyntaxKind {
    #[inline]
    fn from(d: u16) -> SyntaxKind {
        assert!(d <= (SyntaxKind::__LAST as u16));
        unsafe { std::mem::transmute::<u16, SyntaxKind>(d) }
    }
}

impl From<SyntaxKind> for u16 {
    #[inline]
    fn from(k: SyntaxKind) -> u16 {
        k as u16
    }
}

impl SyntaxKind {
    #[inline]
    pub fn is_trivia(self) -> bool {
        matches!(self, SyntaxKind::WHITESPACE | SyntaxKind::COMMENT)
    }
}
