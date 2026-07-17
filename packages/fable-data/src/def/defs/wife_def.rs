use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CWifeDef` — C++ `CWifeDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct WifeDef {
        "Dowry" => pub dowry: DefIndex,
    }
}
