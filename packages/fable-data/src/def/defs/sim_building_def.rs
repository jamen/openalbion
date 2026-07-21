use crate::def_struct;

def_struct! {
    /// `SIM_BUILDING` — C++ `CSimBuildingDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SimBuildingDef {
        "Property" => pub property: i32,
        "WorkSpaces" => pub work_spaces: i32,
        "LiveSpaces" => pub live_spaces: i32,
    }
}
