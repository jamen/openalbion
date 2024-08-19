use crate::{Lexer, Location, Token, TokenKind};

pub struct Tng {
    version: u64,
    sections: Vec<TngSection>,
}

impl Tng {
    // The parser has two stages. The first stage produces a simple AST where every node shares
    // a single `TngNode` type, and expresses the key-value list. The second stage produces a
    // refined AST that reflects the structures found in a Tng file, each node having its own type.
    pub fn parse(source: &str) -> Result<Self, TngParseError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|location| TngParseError {
            location,
            reason: TngParseErrorReason::TokenizerFailure,
        })?;
        let list = Self::parse_stage_one(&tokens)?;
        let tng = Self::parse_stage_two(list)?;
        Ok(tng)
    }
}

pub struct TngSection {
    name: Option<String>,
    items: Vec<TngSectionItem>,
}

pub enum TngSectionItem {
    Thing(TngThing),
    Object(TngObject),
    Marker(TngMarker),
}

pub struct TngThing {}

pub struct TngObject {}

pub struct TngMarker {}

impl Tng {
    fn parse_stage_two(list: TngList) -> Result<Tng, TngParseError> {}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TngParserStageOneState {
    Root,
    Key,
    KeyArrayPart,
    KeyObjectPart,
    Value,
}

struct TngList {
    items: Vec<TngListItem>,
}

struct TngListItem {
    key: TngKey,
    value: TngValue,
}

enum TngKey {
    SinglePart(TngKeyPart),
    MultiPart(Vec<TngKeyPart>),
}

enum TngKeyPart {
    // TODO: Determine the possible key identifiers and turn this into an enum
    Identifier(String),
    // TODO: Determine the possible object index names and turn this into an enum
    ObjectIndex(String),
    ArrayIndex(u64),
}

enum TngValue {
    Number(i64),
    Uid(u64),
    Float(f32),
    String(String),
    // TODO: Determine every possible name and turn it into an enum
    Struct(String, Vec<TngValue>),
    Bool(bool),
    // TODO: Determine the possible identifiers and turn this into an enum
    Identifier(String),
    Empty,
}

pub struct TngParseError {
    pub location: Location,
    pub reason: TngParseErrorReason,
}

pub enum TngParseErrorReason {
    TokenizerFailure,
    UnrecognizedToken,
}

impl Tng {
    fn parse_stage_one<'a>(mut tokens: &[Token]) -> Result<TngList, TngParseError> {
        use TngParserStageOneState as S;
        use TokenKind as T;

        let mut state = TngParserStageOneState::Root;
        let mut current_key = None;
        let mut current_value = TngValue::Empty;
        let mut items = Vec::new();

        loop {
            let (token, rest) = match tokens.split_first() {
                Some(x) => x,
                None => break,
            };

            tokens = rest;

            match (token.text, token.kind, state) {
                (id, T::Identifier, S::Root) => {
                    state = S::Key;
                    current_key = Some(TngKey::SinglePart(TngKeyPart::Identifier(id.to_string())))
                }
                ("[", T::Symbol, S::Key) => {
                    state = S::KeyArrayPart;
                }
                (".", T::Symbol, S::Key) => {
                    state = S::KeyObjectPart;
                }
                (" ", T::Whitespace, S::Key) => {
                    state = S::Value;
                }
                (";", T::Symbol, S::Key) => {
                    state = S::Root;
                    current_value = TngValue::Empty;
                }
                (";", T::Symbol, S::Value) => {
                    state = S::Root;
                }
                _ => {
                    return Err(TngParseError {
                        location: token.location,
                        reason: TngParseErrorReason::UnrecognizedToken,
                    })
                }
            }
        }

        Ok(TngList { items })
    }
}
