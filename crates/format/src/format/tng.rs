use crate::{Lexer, Location, Token};

pub struct TngParser<'a> {
    lexer: Lexer<'a>,
    current_token: Option<Token<'a>>,
    following_token: Option<Token<'a>>,
}

impl<'a> TngParser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            lexer: Lexer::new(source),
            current_token: None,
            following_token: None,
        }
    }

    pub fn parse(&mut self) -> Result<TngRaw, Location> {
        self.following_token = self.lexer.next_token()?;

        loop {
            self.current_token = self.following_token;
            self.following_token = self.lexer.next_token()?;
        }
    }

    fn parse_key(&mut self) -> Result<TngRawKey, Location> {}

    fn parse_value(&mut self) -> Result<TngRawValue, Location> {}
}

pub struct TngRaw {
    pub pairs: Vec<(TngRawKey, TngRawValue)>,
}

pub struct TngRawKey {
    pub path: Vec<TngRawPathPart>,
}

enum TngRawPathPart {
    Identifier(String),
    ArrayIndex(u32),
    ObjectIndex(String),
}

enum TngRawValue {
    None,
    Identifier(String),
    String(String),
    Bool(bool),
    Uid(u64),
    Number(i64),
    Float(f32),
    Coord3D(f32, f32, f32),
}
