use crate::def_struct;

def_struct! {
    /// `COccupiableDef` — C++ `COccupiableDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct OccupiableDef {
        "TypeFlags" => pub type_flags: u32,
    }
}
