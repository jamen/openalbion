use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `THING` — C++ `CThingBaseDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct ThingBaseDef {
        "Components" => pub components: ThingComponentSet,
        "AvailableInEditor" => pub available_in_editor: bool,
        "GameStatePersisted" => pub game_state_persisted: bool,
        "LevelLess" => pub level_less: bool,
        "IsDrawable" => pub is_drawable: bool,
        "AlwaysFrameUpdate" => pub always_frame_update: bool,
        "InMapSearches" => pub in_map_searches: bool,
        "StartKillLocked" => pub start_kill_locked: bool,
        "IsLoadableGlobal" => pub is_loadable_global: bool,
        "CanComeBetweenCameraAndObservedThing" => pub can_come_between_camera_and_observed_thing: bool,
        "IsVulnerableToMelee" => pub is_vulnerable_to_melee: bool,
        "Damageable" => pub damageable: bool,
        "ForceNoSerialise" => pub force_no_serialise: bool,
        "DrawWeaponTrails" => pub draw_weapon_trails: bool,
        "DrawProjectileWeaponTrails" => pub draw_projectile_weapon_trails: bool,
        "AlwaysIncludeInObjectStrikeScans" => pub always_include_in_object_strike_scans: bool,
        "DisableOcclusionTesting" => pub disable_occlusion_testing: bool,
        "DrawAfterWater" => pub draw_after_water: bool,
        "ForceRenderedLastFrameOverride" => pub force_rendered_last_frame_override: bool,
        "AddToComboMultiplierOnHit" => pub add_to_combo_multiplier_on_hit: bool,
        "GiveHeroStatChangesOnBeingHit" => pub give_hero_stat_changes_on_being_hit: bool,
        "GroupDef" => pub group_def: DefIndex,
        "RenderFadeDistance" => pub render_fade_distance: f32,
        "PersistenceFlags" => pub persistence_flags: DefIndex,
        "Health" => pub health: f32,
        "MinHealth" => pub min_health: f32,
        "MeshHeightOffset" => pub mesh_height_offset: f32,
        "MeshHeight" => pub mesh_height: f32,
        "MeshRadius" => pub mesh_radius: f32,
        "ApproxMaxMeshHeight" => pub approx_max_mesh_height: f32,
        "DefaultScriptName" => pub default_script_name: DefString,
        "DefaultScriptData" => pub default_script_data: DefString,
    }
}
