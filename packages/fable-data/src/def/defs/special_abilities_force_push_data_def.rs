use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CSpecialAbilitiesDrainLifeDataDef` | `CSpecialAbilitiesForcePushDataDef` — C++ `CSpecialAbilitiesForcePushDataDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesForcePushDataDef {
        "Damage" => pub damage: f32,
    }
}
