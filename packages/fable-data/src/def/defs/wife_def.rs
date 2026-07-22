use crate::DefStruct;
use crate::def::prelude::*;

/// `CWifeDef` — C++ `CWifeDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct WifeDef {
    #[def("Dowry")]
    pub dowry: DefIndex,
}
