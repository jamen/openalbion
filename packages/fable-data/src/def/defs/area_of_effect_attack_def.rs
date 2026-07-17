use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CAreaOfEffectAttackDef` — C++ `CAreaOfEffectAttackDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct AreaOfEffectAttackDef {
        "TrapIndex" => pub trap_index: i32,
        "ExplosionIndex" => pub explosion_index: i32,
        "ObstructionRadius" => pub obstruction_radius: f32,
        "ObstructionLifeTime" => pub obstruction_life_time: f32,
    }
}
