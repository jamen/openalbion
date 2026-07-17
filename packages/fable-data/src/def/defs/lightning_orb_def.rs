use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CLightningOrbDef` — C++ `CLightningOrbDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct LightningOrbDef {
        "LightningDefIndex" => pub lightning_def_index: i32,
        "LifeTime" => pub life_time: f32,
        "Speed" => pub speed: f32,
        "ExplosionRange" => pub explosion_range: f32,
        "ExplosionDefIndex" => pub explosion_def_index: i32,
    }
}
