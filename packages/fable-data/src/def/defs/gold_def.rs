use crate::DefStruct;

/// `CGoldDef` — C++ `CGoldDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct GoldDef {
    #[def("Gold")]
    pub gold: i32,
}
