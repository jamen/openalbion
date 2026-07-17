use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CBoastingPodiumDef` — C++ `CBoastingPodiumDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct BoastingPodiumDef {
        "HeroOnPodiumRadius" => pub hero_on_podium_radius: f32,
    }
}
