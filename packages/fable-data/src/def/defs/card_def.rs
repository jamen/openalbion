use crate::DefStruct;
use crate::def::prelude::*;

/// C++ `CCardDef` (sub-component def).
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct CardDef {
    #[def("CardName")]
    pub card_name: DefIndex,
    #[def("CardVal")]
    pub card_val: DefIndex,
}
