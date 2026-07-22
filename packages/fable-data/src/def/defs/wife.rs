use crate::DefStruct;
use crate::def::prelude::*;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct WifeDef {
    #[def("Dowry")]
    pub dowry: DefIndex,
}
