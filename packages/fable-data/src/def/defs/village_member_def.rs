use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CVillageMemberDef` — C++ `CVillageMemberDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct VillageMemberDef {
        "NeedsVillageToFunctionCorrectly" => pub needs_village_to_function_correctly: bool,
    }
}
