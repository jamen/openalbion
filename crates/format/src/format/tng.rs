use crate::{Lexer, Location, OwnedToken, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct Tng {
    pub version: u64,
    pub sections: Vec<TngSection>,
}

#[derive(Clone, Debug)]
pub struct TngSection {
    pub name: Option<String>,
    pub items: Vec<TngSectionItem>,
}

#[derive(Clone, Debug)]
pub enum TngSectionItem {
    Thing(TngThing),
    Object(TngObject),
    Marker(TngMarker),
}

#[derive(Clone, Debug)]
pub struct TngThing {}

#[derive(Clone, Debug)]
pub struct TngObject {}

#[derive(Clone, Debug)]
pub struct TngMarker {}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TngParserStageOneState {
    Root,
    Key,
    KeyUid,
    KeyArrayPart,
    KeyObjectPart,
    Value,
    ValueUid,
    ValueIdentifier,
    ValueStructArg,
    ValueStructNextArg,
}

#[derive(Clone, Debug)]
struct TngList {
    items: Vec<TngListItem>,
}

#[derive(Clone, Debug)]
struct TngListItem {
    key: TngKey,
    value: TngValue,
}

#[derive(Clone, Debug)]
enum TngKey {
    SinglePart(TngKeyPart),
    MultiPart(Vec<TngKeyPart>),
}

#[derive(Clone, Debug)]
enum TngKeyPart {
    // TODO: Determine the possible key identifiers and turn this into an enum
    Identifier(String),
    // TODO: Determine the possible object index names and turn this into an enum
    ObjectIndex(String),
    ArrayIndex(u64),
}

#[derive(Clone, Debug)]
enum TngValue {
    Integer(i64),
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

#[derive(Clone, Debug)]
pub struct TngParseError {
    pub location: Location,
    pub token: Option<OwnedToken>,
    pub stage_one_state: Option<TngParserStageOneState>,
    pub reason: TngParseErrorReason,
}

#[derive(Clone, Debug)]
pub enum TngParseErrorReason {
    TokenizeFailed,
    UnexpectedToken,
    /// Parser reached an invalid state. This likely indicates a bug in the parser.
    InvalidState,
    ParseIntError,
    SignedIntError,
}

impl Tng {
    // The parser has two stages. The first stage produces a simple AST where every node shares
    // a single `TngNode` type, and expresses the key-value list. The second stage produces a
    // refined AST that reflects the structures found in a Tng file, each node having its own type.
    pub fn parse(source: &str) -> Result<Self, TngParseError> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().map_err(|location| TngParseError {
            location,
            token: None,
            stage_one_state: None,
            reason: TngParseErrorReason::TokenizeFailed,
        })?;
        let list = Self::parse_stage_one(&tokens)?;
        let tng = Self::parse_stage_two(list)?;
        Ok(tng)
    }

    fn parse_stage_one(mut tokens: &[Token]) -> Result<TngList, TngParseError> {
        use TngParseErrorReason as R;
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
                    current_key = Some(TngKey::SinglePart(TngKeyPart::Identifier(id.to_string())));
                    state = match id {
                        // TODO: Figure out better system for handling UIDs
                        "UID"
                        | "OwnerUID"
                        | "VillageUID"
                        | "ThingToCalculateRouteToUID"
                        | "WorkBuildingUID"
                        | "HomeBuildingUID" => S::KeyUid,
                        _ => S::Key,
                    };
                }
                ("[", T::Symbol, S::Key) => {
                    state = S::KeyArrayPart;
                }
                (index, T::Integer, S::KeyArrayPart) => {
                    let index = match index.parse::<u64>() {
                        Ok(i) => i,
                        Err(_) => {
                            return Err(TngParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                stage_one_state: Some(state),
                                reason: R::ParseIntError,
                            })
                        }
                    };

                    match current_key {
                        Some(TngKey::SinglePart(part)) => {
                            current_key =
                                Some(TngKey::MultiPart(vec![part, TngKeyPart::ArrayIndex(index)]))
                        }
                        Some(TngKey::MultiPart(ref mut parts)) => {
                            parts.push(TngKeyPart::ArrayIndex(index))
                        }
                        None => {
                            return Err(TngParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                stage_one_state: Some(state),
                                reason: R::InvalidState,
                            })
                        }
                    }
                }
                ("]", T::Symbol, S::KeyArrayPart) => {
                    state = S::Key;
                }
                (".", T::Symbol, S::Key) => {
                    state = S::KeyObjectPart;
                }
                (id, T::Identifier, S::KeyObjectPart) => {
                    match current_key {
                        Some(TngKey::SinglePart(part)) => {
                            current_key = Some(TngKey::MultiPart(vec![
                                part,
                                TngKeyPart::ObjectIndex(id.to_string()),
                            ]))
                        }
                        Some(TngKey::MultiPart(ref mut parts)) => {
                            parts.push(TngKeyPart::ObjectIndex(id.to_string()))
                        }
                        None => {
                            return Err(TngParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                stage_one_state: Some(state),
                                reason: R::InvalidState,
                            })
                        }
                    }

                    state = S::Key;
                }

                (" ", T::Whitespace, S::Key) => {
                    state = S::Value;
                }
                (" ", T::Whitespace, S::KeyUid) => {
                    state = S::ValueUid;
                }
                (value, T::Integer, S::Value) => {
                    let value = match value.parse::<i64>() {
                        Ok(x) => x,
                        Err(_) => {
                            return Err(TngParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                stage_one_state: Some(state),
                                reason: R::ParseIntError,
                            })
                        }
                    };

                    current_value = TngValue::Integer(value);
                }
                (value, T::Integer, S::ValueUid) => {
                    let value = match value.parse::<u64>() {
                        Ok(x) => x,
                        Err(_) => {
                            return Err(TngParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                stage_one_state: Some(state),
                                reason: R::ParseIntError,
                            })
                        }
                    };

                    current_value = TngValue::Uid(value);

                    state = S::Value;
                }
                (value, T::Float, S::Value) => {
                    let value = match value.parse::<f32>() {
                        Ok(x) => x,
                        Err(_) => {
                            return Err(TngParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                stage_one_state: Some(state),
                                reason: R::ParseIntError,
                            })
                        }
                    };

                    current_value = TngValue::Float(value);
                }
                (id, T::Identifier, S::Value) => {
                    current_value = TngValue::Identifier(id.to_string());
                }
                ("(", T::Symbol, S::Value) => {
                    let value = std::mem::replace(&mut current_value, TngValue::Empty);

                    let id = match value {
                        TngValue::Identifier(id) => id,
                        _ => {
                            return Err(TngParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                stage_one_state: Some(state),
                                reason: TngParseErrorReason::UnexpectedToken,
                            })
                        }
                    };

                    current_value = TngValue::Struct(id, vec![]);

                    state = S::ValueStructArg;
                }
                (",", T::Symbol, S::ValueStructNextArg) => {
                    state = S::ValueStructArg;
                }
                (value, T::Integer, S::ValueStructArg) => {
                    let value = match value.parse::<i64>() {
                        Ok(x) => x,
                        Err(_) => {
                            return Err(TngParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                stage_one_state: Some(state),
                                reason: R::ParseIntError,
                            })
                        }
                    };

                    match current_value {
                        TngValue::Struct(_, ref mut args) => args.push(TngValue::Integer(value)),
                        _ => {
                            return Err(TngParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                stage_one_state: Some(state),
                                reason: R::InvalidState,
                            })
                        }
                    }

                    state = S::ValueStructNextArg;
                }
                (value, T::Float, S::ValueStructArg) => {
                    let value = match value.parse::<f32>() {
                        Ok(x) => x,
                        Err(_) => {
                            return Err(TngParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                stage_one_state: Some(state),
                                reason: R::ParseIntError,
                            })
                        }
                    };

                    match current_value {
                        TngValue::Struct(_, ref mut args) => args.push(TngValue::Float(value)),
                        _ => {
                            return Err(TngParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                stage_one_state: Some(state),
                                reason: R::InvalidState,
                            })
                        }
                    }

                    state = S::ValueStructNextArg;
                }
                (")", T::Symbol, S::ValueStructNextArg) => {
                    state = S::Value;
                }
                ("\"", T::Symbol, S::Value) => {
                    // Noop
                }
                (string, T::String, S::Value) => {
                    current_value = TngValue::String(string.to_string());
                }
                (";", T::Symbol, S::Key) => {
                    current_value = TngValue::Empty;

                    let key = current_key.take().ok_or_else(|| TngParseError {
                        location: token.location,
                        token: Some(token.to_owned_token()),
                        stage_one_state: Some(state),
                        reason: R::InvalidState,
                    })?;

                    let item = TngListItem {
                        key,
                        value: TngValue::Empty,
                    };

                    items.push(item);

                    state = S::Root
                }
                (";", T::Symbol, S::Value) => {
                    let key = current_key.take().ok_or_else(|| TngParseError {
                        location: token.location,
                        token: Some(token.to_owned_token()),
                        stage_one_state: Some(state),
                        reason: R::InvalidState,
                    })?;

                    let value = std::mem::replace(&mut current_value, TngValue::Empty);

                    let item = TngListItem { key, value };

                    items.push(item);

                    state = S::Root;
                }
                (_, T::Whitespace, _) => {
                    // Ignore whitespace
                }
                _ => {
                    return Err(TngParseError {
                        location: token.location,
                        token: Some(token.to_owned_token()),
                        stage_one_state: Some(state),
                        reason: TngParseErrorReason::UnexpectedToken,
                    })
                }
            }
        }

        Ok(TngList { items })
    }

    fn parse_stage_two(list: TngList) -> Result<Tng, TngParseError> {
        Ok(Tng {
            version: 0,
            sections: vec![],
        })
    }
}
