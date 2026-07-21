use crate::def_struct;

def_struct! {
    /// `CVillageMemberDef` — C++ `CVillageMemberDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct VillageMemberDef {
        "NeedsVillageToFunctionCorrectly" => pub needs_village_to_function_correctly: bool,
    }
}
