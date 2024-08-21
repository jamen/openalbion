mod list;
mod tree;

struct RawTng {
    list: Vec<RawTngPair>,
}

struct RawTngPair {
    key: RawTngKey,
    value: Option<RawTngValue>,
}

struct RawTngKey {
    parts: Vec<RawTngKeyPart>,
}

enum RawTngKeyPart {
    Identifier(RawTngKeyIdentifier),
    ArrayIndex(u64),
    ObjectIndex(RawTngKeyObjectIndex),
    Call,
}

enum RawTngKeyIdentifier {}

enum RawTngKeyObjectIndex {}

enum RawTngValue {
    Integer(i64),
    Uid(u64),
    Float(f32),
    Boolean(bool),
    Identifier(String),
    String(String),
    Struct(String, Vec<RawTngValue>),
}

struct Tng {
    // TODO: Use hashmap?
    sections: Vec<TngSection>,
}

struct TngSection {
    // TODO: Use hashmap?
    items: Vec<TngSectionItem>,
}

enum TngSectionItem {
    Thing(TngThing),
    Marker(TngMarker),
    Object(TngObject),
}

struct TngThing {}

struct TngMarker {}

struct TngObject {}
