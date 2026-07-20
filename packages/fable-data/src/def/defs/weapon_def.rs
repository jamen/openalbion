use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CWeaponDef` — C++ `CWeaponDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct WeaponDef {
        "Type" => pub type_: WeaponType,
        "Class" => pub class: WeaponClass,
        "Property" => pub property: DefIndex,
        "AnimationGroup" => pub animation_group: String = String::from("SWORD"),
        "SheatheObject" => pub sheathe_object: DefIndex,
        "SheatheObjectAlwaysVisible" => pub sheathe_object_always_visible: bool,
        "AnimationSpeedVal" => pub animation_speed_val: f32,
        "Damage" => pub damage: f32,
        "DamageType" => pub damage_type: DamageAttribute = DamageAttribute::MELEE,
        "Explosion" => pub explosion: DefIndex,
        "FlourishParticles" => pub flourish_particles: ObjectAugmentationParticleSet,
        "FlourishSpecialWeaponTrail" => pub flourish_special_weapon_trail: WeaponTrailGraphicSet,
        "WeaponTrails" => pub weapon_trails: VecMap<WeaponTrailGraphicSet, ObjectAugmentationType>,
        "UseAnalogueLoading" => pub use_analogue_loading: bool,
        "MissileLoadCarrySlot" => pub missile_load_carry_slot: DefIndex,
        "ProjectileWeaponHelperNameTip" => pub projectile_weapon_helper_name_tip: String = String::from("weapon_pos_a"),
        "ProjectileWeaponHelperNameBase" => pub projectile_weapon_helper_name_base: String = String::from("weapon_pos_b"),
        "Shot" => pub shot: DefIndex,
        "Ammo" => pub ammo: DefIndex,
        "TargetingRange" => pub targeting_range: DefIndex,
        "TargetingFOV" => pub targeting_fov: f32,
        "ArrowTrails" => pub arrow_trails: VecMap<i32, IdleStateGroup>,
        "ReloadSpeedRange" => pub reload_speed_range: FloatRange,
        "ProjectileWeaponAutoReload" => pub projectile_weapon_auto_reload: bool,
        "ProjectileWeaponStartLoaded" => pub projectile_weapon_start_loaded: bool,
        "UseExtendedProjectileTrails" => pub use_extended_projectile_trails: bool,
        "NextWeaponForBoss" => pub next_weapon_for_boss: i32,
        "MeleeTargetingArc" => pub melee_targeting_arc: f32 = 60.0,
        "NeedsLoading" => pub needs_loading: bool = true,
        "UnsheatheToCombatAnim" => pub unsheathe_to_combat_anim: String,
        "UnsheatheToNormalAnim" => pub unsheathe_to_normal_anim: String,
        "SheatheFromCombatAnim" => pub sheathe_from_combat_anim: String,
        "SheatheFromNormalAnim" => pub sheathe_from_normal_anim: String,
    }
}
