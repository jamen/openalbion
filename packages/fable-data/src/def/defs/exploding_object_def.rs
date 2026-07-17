use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CExplodingObjectDef` — C++ `CExplodingObjectDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ExplodingObjectDef {
        "TriggeredOnCreatureProximity" => pub triggered_on_creature_proximity: bool,
        "TriggerRadius" => pub trigger_radius: f32,
        "ExplosionDef" => pub explosion_def: i32,
        "ProximityParticleEffect" => pub proximity_particle_effect: i32,
    }
}
