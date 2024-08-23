// The parser has two stages. The first stage produces a simple AST where every node shares
// a single `TngNode` type, and expresses the key-value list. The second stage produces a
// refined AST that reflects the structures found in a Tng file, each node having its own type.

use super::list::{TngList, TngListParseError, TngValue, TngValueContents};
use crate::{util::text::Lexer, Location};

#[derive(Clone, Debug)]
pub struct Tng {
    pub sections: Vec<TngSection>,
}

#[derive(Clone, Debug)]
pub struct TngSection {
    pub name: String,
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

#[derive(Clone, Debug)]
pub struct TngParseError {
    pub location: Location,
    pub list_error: Option<TngListParseError>,
    pub kind: TngParseErrorKind,
}

#[derive(Copy, Clone, Debug)]
pub enum TngParseErrorKind {
    TokenizeFailed,
    ListParseError,
    UnexpectedKey,
    UnexpectedVersion,
    ExpectedVersionKey,
    UnexpectedValue,
}
#[derive(Copy, Clone, Debug)]

enum TngParseState {
    Root,
    Section,
}

impl Tng {
    pub fn parse(source: &str) -> Result<Self, TngParseError> {}

    fn parse_list(list: TngList) -> Result<Tng, TngParseError> {
        use TngParseState as S;

        let mut tng = Tng { sections: vec![] };
        let mut section = None;
        // let mut section_item = None;
        let mut state = S::Root;

        for (i, item) in list.items.iter().enumerate() {
            let id = match item.key.identifier() {
                Some(id) => id,
                None => {
                    return Err(TngParseError {
                        location: item.key.location(),
                        list_error: None,
                        kind: TngParseErrorKind::UnexpectedKey,
                    })
                }
            };

            match (id, state) {
                ("Version", S::Root) => {
                    if i == 0 {
                        match item.value {
                            TngValue {
                                contents: TngValueContents::Integer(integer),
                                ..
                            } if integer == 2 => {}
                            TngValue { location, .. } => {
                                return Err(TngParseError {
                                    location,
                                    list_error: None,
                                    kind: TngParseErrorKind::UnexpectedVersion,
                                });
                            }
                        }
                    } else {
                        return Err(TngParseError {
                            location: item.location,
                            list_error: None,
                            kind: TngParseErrorKind::ExpectedVersionKey,
                        });
                    }
                }
                ("XXXSectionStart", S::Root) => {
                    let name = match &item.value.contents {
                        TngValueContents::Identifier(x) => x,
                        _ => {
                            return Err(TngParseError {
                                location: item.location,
                                list_error: None,
                                kind: TngParseErrorKind::UnexpectedValue,
                            });
                        }
                    };

                    section = Some(TngSection {
                        name: name.to_string(),
                        items: vec![],
                    });

                    state = TngParseState::Section;
                }
                ("XXXSectionEnd", S::Section) => {
                    let new_section = match section.take() {
                        Some(section) => section,
                        None => {
                            return Err(TngParseError {
                                location: item.location,
                                list_error: None,
                                kind: TngParseErrorKind::UnexpectedKey,
                            })
                        }
                    };

                    tng.sections.push(new_section);

                    state = S::Root;
                }

                _ => {
                    return Err(TngParseError {
                        location: item.location,
                        list_error: None,
                        kind: TngParseErrorKind::UnexpectedKey,
                    });
                }
            }
        }

        Ok(tng)
    }
}
