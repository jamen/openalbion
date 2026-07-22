use crate::DefStruct;
use crate::def::{
    wire::DefIndex,
};


#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct WifeDef {
    #[def("Dowry")]
    pub dowry: DefIndex,
}
