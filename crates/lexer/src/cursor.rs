use finl_unicode::grapheme_clusters::Graphemes;

pub struct Cursor<'source> {
    token_len: usize,
    graphemes: Graphemes<'source>,
    next_cluster: Option<&'source str>,
    buf: Vec<char>,
}

impl<'a> Cursor<'a> {
    pub fn new(input: &'a str) -> Cursor<'a> {
        let mut graphemes = finl_unicode::grapheme_clusters::Graphemes::new(input);
        let next_cluster = graphemes.next();
        Cursor {
            token_len: 0,
            graphemes,
            next_cluster,
            buf: Vec::new(),
        }
    }

    /// Peeks the next symbol from the input stream without consuming it.
    pub(crate) fn first(&self) -> Option<&str> {
        self.next_cluster
    }

    /// Returns amount of already consumed bytes.
    pub(crate) fn token_len(&self) -> usize {
        self.token_len
    }

    /// Resets the number of bytes consumed to 0.
    pub(crate) fn reset_token_len(&mut self) {
        self.token_len = 0;
    }

    /// Moves to the next grapheme cluster.
    /// Obtain the value by calling self.buf()
    pub(crate) fn bump(&mut self) -> Option<()> {
        let current_cluster = self.next_cluster;
        if let Some(c) = current_cluster {
            self.token_len += c.bytes().len();
        }
        self.next_cluster = self.graphemes.next();

        match current_cluster {
            Some(c) => {
                self.buf = c.chars().collect();
                Some(())
            }
            None => {
                self.buf = Vec::new();
                None
            }
        }
    }

    /// Eats grapheme cluster while predicate returns true or until the end of file is reached.
    pub(crate) fn eat_while(&mut self, mut predicate: impl FnMut(&[char]) -> bool) {
        let mut buf: Vec<char>;

        while self.first().is_some() {
            match self.first() {
                None => break,
                Some(c) => buf = c.chars().collect(),
            };

            if !predicate(&buf) {
                break;
            }
            self.bump();
        }
    }

    pub(crate) fn buf(&self) -> &[char] {
        &self.buf
    }
}
