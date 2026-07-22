use crate::DefStruct;
use crate::def::prelude::*;

/// `CTrapDef` — C++ `CTrapDef`.
#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct TrapDef {
    #[def("Sound")]
    pub sound: VecMap<String, i32>,
    #[def("TriggerType")]
    pub trigger_type: TrapTriggerType,
    #[def("TrapType")]
    pub trap_type: TrapType,
    #[def("TriggerRadius", default = 3.5)]
    pub trigger_radius: f32,
    #[def("TriggeredCollisionMesh")]
    pub triggered_collision_mesh: i32,
    #[def("UntriggeredCollisionMesh")]
    pub untriggered_collision_mesh: i32,
    #[def("TriggerHelperName", default = DefString(-1))]
    pub trigger_helper_name: DefString,
    #[def("TrapDamage")]
    pub trap_damage: f32,
    #[def("ExplosionDefIndex", default = -1)]
    pub explosion_def_index: i32,
    #[def("PhysicalObstructionDefIndex", default = -1)]
    pub physical_obstruction_def_index: i32,
    #[def("NumShotsToFireBeforeIdle", default = 3)]
    pub num_shots_to_fire_before_idle: i32,
    #[def("SecondsIdle", default = 3.0)]
    pub seconds_idle: f32,
    #[def("SecondsBetweenShots", default = 1.0)]
    pub seconds_between_shots: f32,
}
