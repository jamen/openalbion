use crate::def_struct;

def_struct! {
    /// `SPECIAL_ABILITIES_ENFLAME_DEF` — C++ `CSpecialAbilitiesEnflameDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesEnflameDef {
        "LevelUpTimings0" => pub level_up_timings0: f32,
        "LevelUpTimings1" => pub level_up_timings1: f32,
        "LevelUpTimings2" => pub level_up_timings2: f32,
        "Level0Damage" => pub level0_damage: f32,
        "Level1Damage" => pub level1_damage: f32,
        "Level2Damage" => pub level2_damage: f32,
        "Level3Damage" => pub level3_damage: f32,
        "RadiusOfEffect" => pub radius_of_effect: Vec<f32>,
        "Level0CamDist" => pub level0_cam_dist: f32,
        "Level1CamDist" => pub level1_cam_dist: f32,
        "Level2CamDist" => pub level2_cam_dist: f32,
        "Level3CamDist" => pub level3_cam_dist: f32,
        "Level0CamHeight" => pub level0_cam_height: f32,
        "Level1CamHeight" => pub level1_cam_height: f32,
        "Level2CamHeight" => pub level2_cam_height: f32,
        "Level3CamHeight" => pub level3_cam_height: f32,
        "Level0CamFOV" => pub level0_cam_fov: f32,
        "Level1CamFOV" => pub level1_cam_fov: f32,
        "Level2CamFOV" => pub level2_cam_fov: f32,
        "Level3CamFOV" => pub level3_cam_fov: f32,
        "Level0CamSwitchTime" => pub level0_cam_switch_time: f32,
        "Level1CamSwitchTime" => pub level1_cam_switch_time: f32,
        "Level2CamSwitchTime" => pub level2_cam_switch_time: f32,
        "Level3CamSwitchTime" => pub level3_cam_switch_time: f32,
        "EnflameCombatMultiplierFactor" => pub enflame_combat_multiplier_factor: f32,
    }
}
