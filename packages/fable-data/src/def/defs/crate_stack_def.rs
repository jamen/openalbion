use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCrateStackDef` — C++ `CCrateStackDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CrateStackDef {
        "StackType" => pub stack_type: i32,
        "Priority" => pub priority: i32,
    }
}
