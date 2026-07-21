use crate::def_struct;

def_struct! {
    /// `COverheadDisplayDef` — C++ `COverheadDisplayDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct OverheadDisplayDef {
        "DisplayDamage" => pub display_damage: bool,
        "DisplayRepair" => pub display_repair: bool,
        "DisplayRenown" => pub display_renown: bool,
    }
}
