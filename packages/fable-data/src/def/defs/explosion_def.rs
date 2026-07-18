use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CExplosionDef` — C++ `CExplosionDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ExplosionDef {
        "Damage" => pub damage: f32 = 10.0,
        "ExplosionRadius" => pub explosion_radius: f32 = 10.0,
        "FireDamage" => pub fire_damage: i32 = 10,
        "ExplosionLifeTime" => pub explosion_life_time: f32 = 1.5,
        "DamageFalloffWithRadius" => pub damage_falloff_with_radius: bool = true,
        "ExplosionBlendSmall" => pub explosion_blend_small: i32 = -1,
        "ExplosionBlendLarge" => pub explosion_blend_large: i32 = -1,
        "ExplosionReplacementBlendSmall" => pub explosion_replacement_blend_small: i32 = -1,
        "ExplosionReplacementBlendLarge" => pub explosion_replacement_blend_large: i32 = -1,
        "SecondsBeforeEffectReplacement" => pub seconds_before_effect_replacement: f32,
        "ExplosionRadiusSmall" => pub explosion_radius_small: f32,
        "ExplosionRadiusLarge" => pub explosion_radius_large: f32,
        "ExplosionRings" => pub explosion_rings: Vec<ExplosionRing>,
        "ExplosionShouldKnockdown" => pub explosion_should_knockdown: bool = true,
        "CauseFireEffectOnHitCreatures" => pub cause_fire_effect_on_hit_creatures: bool,
        "CausesRumble" => pub causes_rumble: bool,
        "RumbleStrength" => pub rumble_strength: QuakeStrength,
        "RumbleLength" => pub rumble_length: QuakeLength,
    }
}
