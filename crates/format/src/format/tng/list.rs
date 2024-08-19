use crate::util::text::{Location, OwnedToken, Token, TokenKind};

#[derive(Clone, Debug)]
pub struct TngList {
    pub items: Vec<TngListItem>,
}

#[derive(Clone, Debug)]
pub struct TngListItem {
    pub location: Location,
    pub key: TngKey,
    pub value: TngValue,
}

#[derive(Clone, Debug)]
pub enum TngKey {
    SinglePart(TngKeyPart),
    MultiPart(Vec<TngKeyPart>),
}

impl TngKey {
    pub fn first_part<'a>(&'a self) -> &'a TngKeyPart {
        match self {
            TngKey::SinglePart(part) => &part,
            TngKey::MultiPart(parts) => &parts[0],
        }
    }

    pub fn identifier<'a>(&'a self) -> Option<&'a str> {
        match self.first_part() {
            TngKeyPart {
                contents: TngKeyPartContents::Identifier(id),
                ..
            } => Some(&id),
            _ => None,
        }
    }

    pub fn location<'a>(&'a self) -> Location {
        self.first_part().location
    }
}

#[derive(Clone, Debug)]
pub struct TngKeyPart {
    pub location: Location,
    pub contents: TngKeyPartContents,
}

#[derive(Clone, Debug)]
pub enum TngKeyPartContents {
    // TODO: Determine the possible key identifiers and turn this into an enum
    Identifier(String),
    // TODO: Determine the possible object index names and turn this into an enum
    ObjectIndex(String),
    ArrayIndex(u64),
    Call,
}

#[derive(Clone, Debug)]
pub struct TngValue {
    pub location: Location,
    pub contents: TngValueContents,
}

#[derive(Clone, Debug)]
pub enum TngValueContents {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TngListParseState {
    Root,
    Key,
    KeyUid,
    KeyArrayPart,
    KeyObjectPart,
    KeyCall,
    Value,
    ValueUid,
    ValueStructArg,
    ValueStructNextArg,
}

#[derive(Clone, Debug)]
pub struct TngListParseError {
    pub location: Location,
    pub token: Option<OwnedToken>,
    pub state: Option<TngListParseState>,
    pub kind: TngListParseErrorKind,
}

#[derive(Clone, Debug)]
pub enum TngListParseErrorKind {
    TokenizeFailed,
    UnexpectedToken,
    /// Parser reached an invalid state. This likely indicates a bug in the parser.
    InvalidState,
    ParseIntError,
    SignedIntError,
    UnexpectedKey,
    ExpectedVersionKey,
    UnexpectedValue,
}

impl TngList {
    pub fn parse(mut tokens: &[Token]) -> Result<TngList, TngListParseError> {
        use TngListParseErrorKind as R;
        use TngListParseState as S;
        use TokenKind as T;

        let mut state = TngListParseState::Root;
        let mut current_key = None;
        let mut current_value = TngValue {
            location: Location::new(0, 0),
            contents: TngValueContents::Empty,
        };
        let mut items = Vec::new();

        loop {
            let (token, rest) = match tokens.split_first() {
                Some(x) => x,
                None => break,
            };

            tokens = rest;

            match (token.text, token.kind, state) {
                (id, T::Identifier, S::Root) => {
                    current_key = Some(TngKey::SinglePart(TngKeyPart {
                        location: token.location,
                        contents: TngKeyPartContents::Identifier(id.to_string()),
                    }));

                    state = match id {
                        // TODO: Figure out better system for handling UIDs
                        "UID"
                        | "ThingUID"
                        | "OwnerUID"
                        | "VillageUID"
                        | "ThingToCalculateRouteToUID"
                        | "WorkBuildingUID"
                        | "HomeBuildingUID"
                        | "LinkedToUID1"
                        | "LinkedToUID2" => S::KeyUid,
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
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: R::ParseIntError,
                            })
                        }
                    };

                    let key_part = TngKeyPart {
                        location: token.location,
                        contents: TngKeyPartContents::ArrayIndex(index),
                    };

                    match current_key {
                        Some(TngKey::SinglePart(part)) => {
                            current_key = Some(TngKey::MultiPart(vec![part, key_part]))
                        }
                        Some(TngKey::MultiPart(ref mut parts)) => parts.push(key_part),
                        None => {
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: R::InvalidState,
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
                    let key_part = TngKeyPart {
                        location: token.location,
                        contents: TngKeyPartContents::ObjectIndex(id.to_string()),
                    };

                    match current_key {
                        Some(TngKey::SinglePart(part)) => {
                            current_key = Some(TngKey::MultiPart(vec![part, key_part]))
                        }
                        Some(TngKey::MultiPart(ref mut parts)) => parts.push(key_part),
                        None => {
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: R::InvalidState,
                            })
                        }
                    }

                    state = S::Key;
                }
                ("(", T::Symbol, S::Key) => {
                    let key_part = TngKeyPart {
                        location: token.location,
                        contents: TngKeyPartContents::Call,
                    };

                    match current_key {
                        Some(TngKey::SinglePart(part)) => {
                            current_key = Some(TngKey::MultiPart(vec![part, key_part]))
                        }
                        Some(TngKey::MultiPart(ref mut parts)) => parts.push(key_part),
                        None => {
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: R::InvalidState,
                            })
                        }
                    }

                    state = S::KeyCall;
                }
                (")", T::Symbol, S::KeyCall) => {
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
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: R::ParseIntError,
                            })
                        }
                    };

                    current_value = TngValue {
                        location: token.location,
                        contents: TngValueContents::Integer(value),
                    };
                }
                (value, T::Integer, S::ValueUid) => {
                    let value = match value.parse::<u64>() {
                        Ok(x) => x,
                        Err(_) => {
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: R::ParseIntError,
                            })
                        }
                    };

                    current_value = TngValue {
                        location: token.location,
                        contents: TngValueContents::Uid(value),
                    };

                    state = S::Value;
                }
                (value, T::Float, S::Value) => {
                    let value = match value.parse::<f32>() {
                        Ok(x) => x,
                        Err(_) => {
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: R::ParseIntError,
                            })
                        }
                    };

                    current_value = TngValue {
                        location: token.location,
                        contents: TngValueContents::Float(value),
                    };
                }
                (id, T::Identifier, S::Value) => {
                    current_value = TngValue {
                        location: token.location,
                        contents: TngValueContents::Identifier(id.to_string()),
                    };
                }
                ("(", T::Symbol, S::Value) => {
                    let value = std::mem::replace(
                        &mut current_value,
                        TngValue {
                            location: Location::new(0, 0),
                            contents: TngValueContents::Empty,
                        },
                    );

                    let id = match value {
                        TngValue {
                            contents: TngValueContents::Identifier(id),
                            ..
                        } => id,
                        _ => {
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: TngListParseErrorKind::UnexpectedToken,
                            })
                        }
                    };

                    current_value = TngValue {
                        location: token.location,
                        contents: TngValueContents::Struct(id, vec![]),
                    };

                    state = S::ValueStructArg;
                }
                (",", T::Symbol, S::ValueStructNextArg) => {
                    state = S::ValueStructArg;
                }
                (value, T::Integer, S::ValueStructArg) => {
                    let value = match value.parse::<i64>() {
                        Ok(x) => x,
                        Err(_) => {
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: R::ParseIntError,
                            })
                        }
                    };

                    match current_value {
                        TngValue {
                            contents: TngValueContents::Struct(_, ref mut args),
                            ..
                        } => args.push(TngValue {
                            location: token.location,
                            contents: TngValueContents::Integer(value),
                        }),
                        _ => {
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: R::InvalidState,
                            })
                        }
                    }

                    state = S::ValueStructNextArg;
                }
                (value, T::Float, S::ValueStructArg) => {
                    let value = match value.parse::<f32>() {
                        Ok(x) => x,
                        Err(_) => {
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: R::ParseIntError,
                            })
                        }
                    };

                    match current_value {
                        TngValue {
                            contents: TngValueContents::Struct(_, ref mut args),
                            ..
                        } => args.push(TngValue {
                            location: token.location,
                            contents: TngValueContents::Float(value),
                        }),
                        _ => {
                            return Err(TngListParseError {
                                location: token.location,
                                token: Some(token.to_owned_token()),
                                state: Some(state),
                                kind: R::InvalidState,
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
                    current_value = TngValue {
                        location: token.location,
                        contents: TngValueContents::String(string.to_string()),
                    };
                }
                (";", T::Symbol, S::Key) => {
                    current_value = TngValue {
                        location: Location::new(0, 0),
                        contents: TngValueContents::Empty,
                    };

                    let key = current_key.take().ok_or_else(|| TngListParseError {
                        location: token.location,
                        token: Some(token.to_owned_token()),
                        state: Some(state),
                        kind: R::InvalidState,
                    })?;

                    let location = match &key {
                        TngKey::SinglePart(key_part) => key_part.location,
                        TngKey::MultiPart(key_parts) => key_parts[0].location,
                    };

                    let item = TngListItem {
                        location,
                        key,
                        value: TngValue {
                            location: token.location,
                            contents: TngValueContents::Empty,
                        },
                    };

                    items.push(item);

                    state = S::Root
                }
                (";", T::Symbol, S::Value) => {
                    let key = current_key.take().ok_or_else(|| TngListParseError {
                        location: token.location,
                        token: Some(token.to_owned_token()),
                        state: Some(state),
                        kind: R::InvalidState,
                    })?;

                    let value = std::mem::replace(
                        &mut current_value,
                        TngValue {
                            location: Location::new(0, 0),
                            contents: TngValueContents::Empty,
                        },
                    );

                    let location = match &key {
                        TngKey::SinglePart(key_part) => key_part.location,
                        TngKey::MultiPart(key_parts) => key_parts[0].location,
                    };

                    let item = TngListItem {
                        location,
                        key,
                        value,
                    };

                    items.push(item);

                    state = S::Root;
                }
                (_, T::Whitespace, _) => {
                    // Ignore whitespace
                }
                _ => {
                    return Err(TngListParseError {
                        location: token.location,
                        token: Some(token.to_owned_token()),
                        state: Some(state),
                        kind: TngListParseErrorKind::UnexpectedToken,
                    })
                }
            }
        }

        Ok(TngList { items })
    }
}
