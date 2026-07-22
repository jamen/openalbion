use crate::DefStruct;

/// `CDragonActionHoverDef` — C++ `CDragonActionHoverDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct DragonActionHoverDef {
    #[def("WindSpeed")]
    pub wind_speed: f32,
}
