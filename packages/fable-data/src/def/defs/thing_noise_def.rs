use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `NOISE` — C++ `CThingNoiseDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ThingNoiseDef {
        "Components" => pub components: ThingComponentSet,
        "AvailableInEditor" => pub available_in_editor: bool = true,
        "GameStatePersisted" => pub game_state_persisted: bool,
        "LevelLess" => pub level_less: bool,
        "IsDrawable" => pub is_drawable: bool,
        "AlwaysFrameUpdate" => pub always_frame_update: bool = true,
        "InMapSearches" => pub in_map_searches: bool = true,
        "StartKillLocked" => pub start_kill_locked: bool,
        "IsLoadableGlobal" => pub is_loadable_global: bool,
        "CanComeBetweenCameraAndObservedThing" => pub can_come_between_camera_and_observed_thing: bool = true,
        "IsVulnerableToMelee" => pub is_vulnerable_to_melee: bool,
        "Damageable" => pub damageable: bool,
        "ForceNoSerialise" => pub force_no_serialise: bool,
        "DrawWeaponTrails" => pub draw_weapon_trails: bool = true,
        "DrawProjectileWeaponTrails" => pub draw_projectile_weapon_trails: bool = true,
        "AlwaysIncludeInObjectStrikeScans" => pub always_include_in_object_strike_scans: bool,
        "DisableOcclusionTesting" => pub disable_occlusion_testing: bool,
        "DrawAfterWater" => pub draw_after_water: bool,
        "ForceRenderedLastFrameOverride" => pub force_rendered_last_frame_override: bool,
        "AddToComboMultiplierOnHit" => pub add_to_combo_multiplier_on_hit: bool,
        "GiveHeroStatChangesOnBeingHit" => pub give_hero_stat_changes_on_being_hit: bool,
        "GroupDef" => pub group_def: DefIndex,
        "RenderFadeDistance" => pub render_fade_distance: f32 = 64.0,
        "PersistenceFlags" => pub persistence_flags: DefIndex,
        "Health" => pub health: f32,
        "MinHealth" => pub min_health: f32 = -0.1,
        "MeshHeightOffset" => pub mesh_height_offset: f32,
        "MeshHeight" => pub mesh_height: f32 = 2.0,
        "MeshRadius" => pub mesh_radius: f32 = 1.0,
        "ApproxMaxMeshHeight" => pub approx_max_mesh_height: f32 = 2.0,
        "DefaultScriptName" => pub default_script_name: DefString,
        "DefaultScriptData" => pub default_script_data: DefString,
        "Noise" => pub noise: TCNoiseDef,
    }
}
