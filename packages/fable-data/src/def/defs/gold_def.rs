use crate::def_struct;

def_struct! {
    /// `CGoldDef` — C++ `CGoldDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct GoldDef {
        "Gold" => pub gold: i32,
    }
}
