use crate::DefStruct;

/// `CSpecialAbilitiesDrainLifeDataDef` | `CSpecialAbilitiesForcePushDataDef` — C++ `CSpecialAbilitiesForcePushDataDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct SpecialAbilitiesForcePushDataDef {
    #[def("Damage")]
    pub damage: f32,
}
