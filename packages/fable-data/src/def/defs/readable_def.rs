use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CReadableDef` — C++ `CReadableDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ReadableDef {
        "UseInfoDisplay" => pub use_info_display: bool,
    }
}
