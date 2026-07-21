use crate::def_struct;

def_struct! {
    /// `CHairCardDef` — C++ `CHairCardDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HairCardDef {
        "HairObject" => pub hair_object: i32,
    }
}
