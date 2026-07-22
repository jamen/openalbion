use crate::DefStruct;

/// `CReadableDef` — C++ `CReadableDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct ReadableDef {
    #[def("UseInfoDisplay")]
    pub use_info_display: bool,
}
