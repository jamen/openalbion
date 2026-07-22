use crate::DefStruct;

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct AreaOfEffectAttackDef {
    #[def("TrapIndex")]
    pub trap_index: i32,
    #[def("ExplosionIndex")]
    pub explosion_index: i32,
    #[def("ObstructionRadius")]
    pub obstruction_radius: f32,
    #[def("ObstructionLifeTime")]
    pub obstruction_life_time: f32,
}
