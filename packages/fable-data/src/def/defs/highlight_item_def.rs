use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CHighlightItemDef` — C++ `CHighlightItemDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HighlightItemDef {
        "StartAsRunning" => pub start_as_running: bool,
        "ParticleEmitter" => pub particle_emitter: i32,
        "PickupParticleEmitter" => pub pickup_particle_emitter: i32,
        "RotationSpeed" => pub rotation_speed: f32,
        "DistanceFromGround" => pub distance_from_ground: f32,
        "BobTime" => pub bob_time: f32,
        "BobDistance" => pub bob_distance: f32,
        "StartAsRunningDelay" => pub start_as_running_delay: f32,
    }
}
