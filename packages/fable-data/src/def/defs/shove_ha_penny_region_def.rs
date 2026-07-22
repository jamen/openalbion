use crate::DefStruct;

/// C++ `CShoveHaPennyRegionDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ShoveHaPennyRegionDef {
    #[def("Start")]
    pub start: f32,
    #[def("Score")]
    pub score: f32,
}
