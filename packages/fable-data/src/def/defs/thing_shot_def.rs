use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SHOT` — C++ `CThingShotDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ThingShotDef {
        "Speed" => pub speed: f32,
        "PrimaryEffect" => pub primary_effect: i32,
        "PrimaryEffectSound" => pub primary_effect_sound: i32,
        "AttachedContinuousEffect" => pub attached_continuous_effect: i32,
        "AttachedContinuousSound" => pub attached_continuous_sound: i32,
        "UseEndTimeLineOnContinousEffect" => pub use_end_time_line_on_continous_effect: bool,
        "HitThingDamage" => pub hit_thing_damage: f32,
        "HitThingShieldDamage" => pub hit_thing_shield_damage: f32,
        "CauseDamage" => pub cause_damage: bool,
        "ShotDefinitionClassName" => pub shot_definition_class_name: DefString,
        "ShotOffsetAngleRange" => pub shot_offset_angle_range: FloatRange,
        "ShotOffsetRadiusRange" => pub shot_offset_radius_range: FloatRange,
        "ExplosionDef" => pub explosion_def: i32,
        "GroundExplosionDef" => pub ground_explosion_def: i32,
        "ObjectDef" => pub object_def: i32,
        "LeavesObjectBehind" => pub leaves_object_behind: bool,
        "IsHittableByMelee" => pub is_hittable_by_melee: bool,
        "IsDeflectedOnHitByMelee" => pub is_deflected_on_hit_by_melee: bool,
        "MeleeHitDeflectionDamageMultiplier" => pub melee_hit_deflection_damage_multiplier: f32,
        "FaceMovementDirection" => pub face_movement_direction: bool,
        "AllowDeflections" => pub allow_deflections: bool,
        "BounceOnGround" => pub bounce_on_ground: bool,
        "Bounciness" => pub bounciness: f32,
        "RotateOnDeflection" => pub rotate_on_deflection: bool,
        "IsBlockable" => pub is_blockable: bool,
        "DamageType" => pub damage_type: DamageAttribute,
        "NumSubShots" => pub num_sub_shots: i32,
        "SubShotType" => pub sub_shot_type: i32,
        "SubShotCreationTime" => pub sub_shot_creation_time: f32,
        "SubShotMaxAngle" => pub sub_shot_max_angle: f32,
        "RemoveMainShotOnSubShotCreation" => pub remove_main_shot_on_sub_shot_creation: bool,
        "DoesExplosionDamageCaster" => pub does_explosion_damage_caster: bool,
        "CanHitRollingTargets" => pub can_hit_rolling_targets: bool,
        "PreventSlidingOnCollision" => pub prevent_sliding_on_collision: bool,
    }
}
