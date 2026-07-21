use crate::def_struct;

def_struct! {
    /// `CSpecialAbilitiesDrainLifeDataDef` | `CSpecialAbilitiesForcePushDataDef` — C++ `CSpecialAbilitiesForcePushDataDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesForcePushDataDef {
        "Damage" => pub damage: f32,
    }
}
