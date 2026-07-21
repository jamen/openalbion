use crate::def_struct;

def_struct! {
    /// `CCrateStackDef` — C++ `CCrateStackDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CrateStackDef {
        "StackType" => pub stack_type: i32,
        "Priority" => pub priority: i32,
    }
}
