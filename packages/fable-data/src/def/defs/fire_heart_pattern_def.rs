use crate::DefStruct;

/// C++ `CFireHeartPatternDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct FireHeartPatternDef {
    #[def("PadActive")]
    pub pad_active: Vec<bool>,
}
