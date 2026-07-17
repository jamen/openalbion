use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `MATERIAL` — C++ `CMaterialDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct MaterialDef {
        "FirstHitSound" => pub first_hit_sound: i32,
        "LastHitSound" => pub last_hit_sound: i32,
        "OnHitRumbleLevel" => pub on_hit_rumble_level: f32,
        "OnHitRumbleFalloff" => pub on_hit_rumble_falloff: f32,
        "DeflectAllHits" => pub deflect_all_hits: bool,
        "IsDiggable" => pub is_diggable: bool,
        "SlideFootParticleEffect" => pub slide_foot_particle_effect: Vec<i32>,
        "SwordHitParticleEffect" => pub sword_hit_particle_effect: Vec<i32>,
        "KnockdownHitFloorParticleEffect" => pub knockdown_hit_floor_particle_effect: Vec<i32>,
        "WindBlowDustEffect" => pub wind_blow_dust_effect: Vec<i32>,
        "BerserkedSwordHitParticleEffect" => pub berserked_sword_hit_particle_effect: Vec<i32>,
        "BlendedSwordHitEffects" => pub blended_sword_hit_effects: VecMap<i32, BlendedParticleEffectSet>,
    }
}
