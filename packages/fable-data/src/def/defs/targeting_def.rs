use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CTargetingDef` — C++ `CTargetingDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct TargetingDef {
        "Type" => pub type_: u32,
        "UseGlowTargeting" => pub use_glow_targeting: bool,
        "TargetArc" => pub target_arc: f32,
        "TargetArcPCThirdPersonAimingMode" => pub target_arc_pc_third_person_aiming_mode: f32,
        "TargetingBasePosFromCamera" => pub targeting_base_pos_from_camera: bool,
        "TargetingFacingDirFromCamera" => pub targeting_facing_dir_from_camera: bool,
        "TargetingPCFacingDirFromCamera" => pub targeting_pc_facing_dir_from_camera: bool,
        "RejectTargetsBehindIfRunning" => pub reject_targets_behind_if_running: bool,
        "AcceptOnlyEnemiesBehind" => pub accept_only_enemies_behind: bool,
        "EnemyBehindTargetingRange" => pub enemy_behind_targeting_range: f32,
        "FirstPersonTargetingAngleFalloff" => pub first_person_targeting_angle_falloff: f32,
        "FirstPersonTargetingDistanceFalloff" => pub first_person_targeting_distance_falloff: f32,
        "ThirdPersonTargetingAngleFalloff" => pub third_person_targeting_angle_falloff: f32,
        "ThirdPersonTargetingDistanceFalloff" => pub third_person_targeting_distance_falloff: f32,
        "ZLockTargetSelectAngleFalloff" => pub z_lock_target_select_angle_falloff: f32,
        "ZLockTargetSelectDistanceFalloff" => pub z_lock_target_select_distance_falloff: f32,
        "FirstPersonTargetingOverrideShortRangeFactor" => pub first_person_targeting_override_short_range_factor: f32,
        "FirstPersonTargetingOverrideShortRangeFalloff" => pub first_person_targeting_override_short_range_falloff: f32,
        "ThirdPersonTargetingOverrideShortRangeFactor" => pub third_person_targeting_override_short_range_factor: f32,
        "ThirdPersonTargetingOverrideShortRangeFalloff" => pub third_person_targeting_override_short_range_falloff: f32,
        "FirstPersonTargetingEnemyPreferenceFactor" => pub first_person_targeting_enemy_preference_factor: f32,
        "ThirdPersonTargetingEnemyPreferenceFactor" => pub third_person_targeting_enemy_preference_factor: f32,
        "TargetingRanges" => pub targeting_ranges: VecMap<f32, Opinion>,
        "PreferPlayerWeight" => pub prefer_player_weight: f32,
        "PreferNonCombatantsWeight" => pub prefer_non_combatants_weight: f32,
        "PreferLastAttackerWeight" => pub prefer_last_attacker_weight: f32,
        "PreferCurrentBestEnemyWeight" => pub prefer_current_best_enemy_weight: f32,
    }
}
