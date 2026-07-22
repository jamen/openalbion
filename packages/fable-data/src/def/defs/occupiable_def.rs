use crate::DefStruct;

/// `COccupiableDef` — C++ `COccupiableDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct OccupiableDef {
    #[def("TypeFlags")]
    pub type_flags: u32,
}
