use crate::{Lexer, Location, Token};

pub struct Tng {
    version: u64,
    sections: Vec<TngSection>,
}

impl Tng {
    // The parser has two stages. The first stage produces a simple AST where every node shares
    // a single `TngNode` type, and expresses the key-value list. The second stage produces a
    // refined AST that reflects the structures found in a Tng file, each node having its own type.
    fn parse(source: &str) -> Result<Self, Location> {}
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

struct TngParserStageOne {
    state: TngParserStageOneState,
    current_node: Option<TngNode>,
    list: Vec<(TngKey, TngValue)>,
}

impl TngParserStageOne {
    fn new() -> Self {
        Self {
            state: TngParserStageOneState::Root,
            current_node: None,
            list: Vec::new(),
        }
    }
}

enum TngParserStageOneState {
    Root,
}

enum TngNode {
    Key(TngKey),
    Value(TngValue),
}

enum TngKey {
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

struct TngParserStageTwo {
    root: Tng,
}
