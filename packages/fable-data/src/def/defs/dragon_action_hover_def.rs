use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CDragonActionHoverDef` — C++ `CDragonActionHoverDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct DragonActionHoverDef {
        "WindSpeed" => pub wind_speed: f32,
    }
}
