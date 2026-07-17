use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CExplosionDef` — C++ `CExplosionDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ExplosionDef {
        "Damage" => pub damage: f32,
        "ExplosionRadius" => pub explosion_radius: f32,
        "FireDamage" => pub fire_damage: i32,
        "ExplosionLifeTime" => pub explosion_life_time: f32,
        "DamageFalloffWithRadius" => pub damage_falloff_with_radius: bool,
        "ExplosionBlendSmall" => pub explosion_blend_small: i32,
        "ExplosionBlendLarge" => pub explosion_blend_large: i32,
        "ExplosionReplacementBlendSmall" => pub explosion_replacement_blend_small: i32,
        "ExplosionReplacementBlendLarge" => pub explosion_replacement_blend_large: i32,
        "SecondsBeforeEffectReplacement" => pub seconds_before_effect_replacement: f32,
        "ExplosionRadiusSmall" => pub explosion_radius_small: f32,
        "ExplosionRadiusLarge" => pub explosion_radius_large: f32,
        "ExplosionRings" => pub explosion_rings: Vec<ExplosionRing>,
        "ExplosionShouldKnockdown" => pub explosion_should_knockdown: bool,
        "CauseFireEffectOnHitCreatures" => pub cause_fire_effect_on_hit_creatures: bool,
        "CausesRumble" => pub causes_rumble: bool,
        "RumbleStrength" => pub rumble_strength: QuakeStrength,
        "RumbleLength" => pub rumble_length: QuakeLength,
    }
}
