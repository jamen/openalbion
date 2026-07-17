use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CBuyHouseDef` — C++ `CBuyHouseDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BuyHouseDef {
        "Main" => pub main: bool,
    }
}
