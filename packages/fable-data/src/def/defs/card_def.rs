use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// C++ `CCardDef` (sub-component def).
    #[derive(Debug, Clone, PartialEq)]
    pub struct CardDef {
        "CardName" => pub card_name: DefIndex,
        "CardVal" => pub card_val: DefIndex,
    }
}
