use crate::def_struct;

def_struct! {
    /// `SPECIAL_ABILITIES_FORCE_PUSH_DEF` — C++ `CSpecialAbilitiesForcePushDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesForcePushDef {
        "MaxDistanceToPush" => pub max_distance_to_push: f32,
        "MinDistanceToPush" => pub min_distance_to_push: f32,
        "DustBoxWidth" => pub dust_box_width: f32,
        "MaxNumOfExtraEffects" => pub max_num_of_extra_effects: i32,
        "MinNumOfExtraEffects" => pub min_num_of_extra_effects: i32,
        "LevelUpTimings0" => pub level_up_timings0: f32,
        "LevelUpTimings1" => pub level_up_timings1: f32,
        "LevelUpTimings2" => pub level_up_timings2: f32,
        "ReleaseLevel1Delay" => pub release_level1_delay: f32,
        "ReleaseLevel2Delay" => pub release_level2_delay: f32,
        "ReleaseLevel3DelayA" => pub release_level3_delay_a: f32,
        "ReleaseLevel3DelayB" => pub release_level3_delay_b: f32,
        "ReleaseLevel4Delay" => pub release_level4_delay: f32,
        "HeightOffGround" => pub height_off_ground: f32,
        "Level4OffsetMagnitude" => pub level4_offset_magnitude: f32,
        "RadiusOfEffect" => pub radius_of_effect: Vec<f32>,
        "ExtraPush" => pub extra_push: f32,
        "PushSpeed" => pub push_speed: f32,
        "ObjectDamage" => pub object_damage: Vec<f32>,
        "TimeDelayBetweenCastsSec" => pub time_delay_between_casts_sec: Vec<f32>,
    }
}
