use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTrapDef` — C++ `CTrapDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TrapDef {
        "Sound" => pub sound: VecMap<String, i32>,
        "TriggerType" => pub trigger_type: TrapTriggerType,
        "TrapType" => pub trap_type: TrapType,
        "TriggerRadius" => pub trigger_radius: f32,
        "TriggeredCollisionMesh" => pub triggered_collision_mesh: DefIndex,
        "UntriggeredCollisionMesh" => pub untriggered_collision_mesh: DefIndex,
        "TriggerHelperName" => pub trigger_helper_name: DefString,
        "TrapDamage" => pub trap_damage: f32,
        "ExplosionDefIndex" => pub explosion_def_index: DefIndex,
        "PhysicalObstructionDefIndex" => pub physical_obstruction_def_index: DefIndex,
        "NumShotsToFireBeforeIdle" => pub num_shots_to_fire_before_idle: DefIndex,
        "SecondsIdle" => pub seconds_idle: f32,
        "SecondsBetweenShots" => pub seconds_between_shots: f32,
    }
}
