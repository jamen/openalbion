use crate::util::{
    slice::TakeSliceExt,
    text::{Lexer, Location, Token, TokenKind},
};

pub struct Tng {
    // TODO: Use hashmap?
    sections: Vec<TngSection>,
}

pub struct TngSection {
    // TODO: Use hashmap?
    items: Vec<TngSectionItem>,
}

pub enum TngSectionItem {
    Thing(TngThing),
    Marker(TngMarker),
    Object(TngObject),
}

pub struct TngThing {}

pub struct TngMarker {}

pub struct TngObject {}

pub struct TngParseError {}

enum TngParseState {}

impl Tng {
    pub fn parse(source: &str) -> Result<Tng, TngParseError> {
        let tokens = Lexer::tokenize(source).map_err(|_loc| TngParseError {})?;

        let raw_tng = RawTng::parse(&tokens).map_err(|_err| TngParseError {})?;

        Self::parse_raw_tng(raw_tng)
    }

    fn parse_raw_tng(raw_tng: RawTng) -> Result<Tng, TngParseError> {}
}

struct RawTng {
    list: Vec<RawTngPair>,
}

impl RawTng {
    fn parse(mut tokens: &[Token]) -> Result<RawTng, RawTngParseError> {
        let mut raw_tng = RawTng { list: vec![] };

        while !tokens.is_empty() {
            let pair = RawTngPair::parse(&mut tokens)?;
            raw_tng.list.push(pair);
        }

        Ok(raw_tng)
    }
}

struct RawTngPair {
    key: RawTngKey,
    value: Option<RawTngValue>,
}

impl RawTngPair {
    fn parse(mut tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        let key = RawTngKey::parse(&mut tokens)?;
        let value = RawTngValue::parse(&mut tokens)?;
        Ok(RawTngPair { key, value })
    }
}

struct RawTngKey {
    ident: RawTngKeyIdentifier,
    indices: Vec<RawTngKeyIndex>,
}

impl RawTngKey {
    fn parse(mut tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let ident = RawTngKeyIdentifier::parse(&mut tokens)?;

        let mut indices = vec![];

        loop {
            let index_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
                location: None,
                kind: E::UnexpectedEOF,
            })?;
        }

        Ok(RawTngKey { ident, indices })
    }
}

enum RawTngKeyIdentifier {
    Version,
}

impl RawTngKeyIdentifier {
    fn parse(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        use RawTngKeyIdentifier as I;
        use RawTngParseErrorKind as E;
        use TokenKind as T;

        let ident_token = match tokens.grab_first() {
            Some(token) => token,
            None => {
                return Err(RawTngParseError {
                    location: None,
                    kind: E::UnexpectedEOF,
                })
            }
        };

        if ident_token.kind != T::Identifier {
            return Err(RawTngParseError {
                location: Some(ident_token.location),
                kind: E::UnexpectedToken,
            });
        }

        Ok(match ident_token.text {
            "Version" => I::Version,
            _ => {
                return Err(RawTngParseError {
                    location: Some(ident_token.location),
                    kind: E::UnrecognizedIdentifier,
                })
            }
        })
    }
}

enum RawTngKeyIndex {
    Array(u64),
    Property(RawTngKeyProperty),
    Call,
}

impl RawTngKeyIndex {
    fn parse(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {
        match (index_token.kind, index_token.text) {
            (T::Symbol, ".") => {}
            (T::Identifier, raw_ident) => {
                let property = RawTngKeyProperty::new(raw_ident).ok_or_else(|| RawTngParseError {
                    location: Some(index_token.location),
                    kind: E::UnrecognizedIdentifier,
                });
            }
            (T::Symbol, "[") => {
                let integer_token = tokens.grab_first().ok_or_else(|| RawTngParseError {
                    location: None,
                    kind: E::UnexpectedEOF,
                });

                match (integer_token.kind, integer_token.text) {
                    (T::Integer, integer_text) => {}
                }
            }
            (T::Whitespace, " ") => break,
            _ => {
                return Err(RawTngParseError {
                    location: Some(index_token.location),
                    kind: E::UnexpectedToken,
                })
            }
        }
    }

    fn parse_property(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {}

    fn parse_array(tokens: &mut &[Token]) -> Result<Self, RawTngParseError> {}
}

enum RawTngKeyProperty {}

enum RawTngValue {
    Integer(i32),
    Uid(u64),
    Float(f32),
    Boolean(bool),
    Identifier(String),
    String(String),
    Struct(String, Vec<RawTngValue>),
}

impl RawTngValue {
    fn parse(tokens: &mut &[Token]) -> Result<Option<Self>, RawTngParseError> {
        let working_value = None;

        Ok(working_value)
    }
}

struct RawTngParseError {
    location: Option<Location>,
    kind: RawTngParseErrorKind,
}

enum RawTngParseErrorKind {
    UnexpectedEOF,
    UnexpectedToken,
    UnrecognizedIdentifier,
}
