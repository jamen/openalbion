use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTrapDef` — C++ `CTrapDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TrapDef {
        "Sound" => pub sound: VecMap<String, i32>,
        "TriggerType" => pub trigger_type: TrapTriggerType,
        "TrapType" => pub trap_type: TrapType,
        "TriggerRadius" => pub trigger_radius: f32 = 3.5,
        "TriggeredCollisionMesh" => pub triggered_collision_mesh: i32,
        "UntriggeredCollisionMesh" => pub untriggered_collision_mesh: i32,
        "TriggerHelperName" => pub trigger_helper_name: DefString = DefString(-1),
        "TrapDamage" => pub trap_damage: f32,
        "ExplosionDefIndex" => pub explosion_def_index: i32 = -1,
        "PhysicalObstructionDefIndex" => pub physical_obstruction_def_index: i32 = -1,
        "NumShotsToFireBeforeIdle" => pub num_shots_to_fire_before_idle: i32 = 3,
        "SecondsIdle" => pub seconds_idle: f32 = 3.0,
        "SecondsBetweenShots" => pub seconds_between_shots: f32 = 1.0,
    }
}
