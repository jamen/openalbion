use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `REGION` — C++ `CRegionDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct RegionDef {
        "EnvironmentTheme" => pub environment_theme: DefIndex,
        "InternalEnvironmentTheme" => pub internal_environment_theme: DefIndex,
        "InternalLightingChannelCount" => pub internal_lighting_channel_count: i32 = 1,
        "SoundMap" => pub sound_map: SoundMap,
        "PreventHeroWeaponUse" => pub prevent_hero_weapon_use: bool,
        "PreventHeroWillUse" => pub prevent_hero_will_use: bool,
        "WanderingPopulationScriptDefName" => pub wandering_population_script_def_name: DefString,
        "Music" => pub music: i32,
        "ReverbEnvironmentType" => pub reverb_environment_type: ReverbEnvironmentType = ReverbEnvironmentType::EXTERNAL,
        "ReverbEnvironmentLevel" => pub reverb_environment_level: f32 = 1.0,
        "InBuildingOcclusionLevel" => pub in_building_occlusion_level: f32 = 1.0,
        "UseVillageTime" => pub use_village_time: bool,
        "AllowHenchmen" => pub allow_henchmen: bool = true,
        "AttachToCameraFX" => pub attach_to_camera_fx: i32,
        "EnableSpotlightShadowMode" => pub enable_spotlight_shadow_mode: bool,
        "CheckAllThingsOnMapForCombatCollision" => pub check_all_things_on_map_for_combat_collision: bool,
        "HeroBreathParticle" => pub hero_breath_particle: i32,
        "MaxNumAttackersOnHero" => pub max_num_attackers_on_hero: i32 = -1,
        "CameraManagerSetIndex" => pub camera_manager_set_index: DefIndex = DefIndex(-1),
        "CameraManagerSetBuildingIndex" => pub camera_manager_set_building_index: DefIndex = DefIndex(-1),
        "RegionDescription" => pub region_description: i32,
        "WeaponOutCrime" => pub weapon_out_crime: bool,
        "UseCrimes" => pub use_crimes: bool,
        "DisplayCrimes" => pub display_crimes: bool,
        "PermitCrimes" => pub permit_crimes: bool,
        "HasSea" => pub has_sea: bool,
        "LockToFrameRate" => pub lock_to_frame_rate: i32 = 30,
        "WorldMapOffsetX" => pub world_map_offset_x: f32,
        "WorldMapOffsetY" => pub world_map_offset_y: f32,
        "NameGraphicOffsetX" => pub name_graphic_offset_x: f32,
        "NameGraphicOffsetY" => pub name_graphic_offset_y: f32,
        "NameGraphic" => pub name_graphic: WorldMapNameGraphic,
        "FishWeightMult" => pub fish_weight_mult: f32 = 1.0,
    }
}
