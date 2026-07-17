use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CHeroExperienceDef` — C++ `CHeroExperienceDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct HeroExperienceDef {
        "ExperienceLevelThresholds" => pub experience_level_thresholds: Vec<i32>,
        "HeroLevelTitles" => pub hero_level_titles: Vec<i32>,
        "FallbackMultipliers" => pub fallback_multipliers: Vec<i32>,
        "SoundIntensityThresholds" => pub sound_intensity_thresholds: VecMap<i32, DefString>,
        "ComboMultiplierMap" => pub combo_multiplier_map: VecMap<i32, ComboMultiplierData>,
        "ExperienceMultiplierPerUnitComboMultiplierParam" => pub experience_multiplier_per_unit_combo_multiplier_param: f32,
        "ExperienceMultiplierRegionChangeMultiplier" => pub experience_multiplier_region_change_multiplier: f32,
        "StatIncreasePerHitType" => pub stat_increase_per_hit_type: VecMap<DamageAttribute, HeroStatIncreaseData>,
        "ComboMultiplierIncrease" => pub combo_multiplier_increase: VecMap<i32, DefString>,
        "ComboMultiplierLock" => pub combo_multiplier_lock: VecMap<i32, DefString>,
        "ComboMultiplierLost" => pub combo_multiplier_lost: VecMap<i32, DefString>,
        "ComboMultiplierRunningHitsIncrease" => pub combo_multiplier_running_hits_increase: VecMap<i32, DefString>,
        "StatIncreasePerHitType" => pub stat_increase_per_hit_type2: VecMap<DamageAttribute, HeroStatIncreaseData>,
        "AttackHistoryCombos" => pub attack_history_combos: Vec<AttackHistoryCombo>,
    }
}
