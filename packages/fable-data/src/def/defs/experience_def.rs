use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CExperienceDef` — C++ `CExperienceDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ExperienceDef {
        "OrbCombiningParticleEffect" => pub orb_combining_particle_effect: i32,
        "ParticleScalesPerExperiencePoints" => pub particle_scales_per_experience_points: VecMap<i32, f32>,
        "PickupParticleEffectsPerExperiencePoints" => pub pickup_particle_effects_per_experience_points: VecMap<i32, i32>,
        "LifeTimeForDistanceThresholds" => pub life_time_for_distance_thresholds: VecMap<i32, f32>,
    }
}
