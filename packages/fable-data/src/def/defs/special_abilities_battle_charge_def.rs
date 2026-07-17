use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_BATTLE_CHARGE_DEF` — C++ `CSpecialAbilitiesBattleChargeDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesBattleChargeDef {
        "RunSpeed" => pub run_speed: f32,
        "DashDistance" => pub dash_distance: Vec<f32>,
        "RadiusOfEffect" => pub radius_of_effect: Vec<f32>,
        "BattleChargeDamage" => pub battle_charge_damage: Vec<f32>,
        "BattleChargeCombatMultiplierFactor" => pub battle_charge_combat_multiplier_factor: f32,
        "BattleChargeRadialStartSeconds" => pub battle_charge_radial_start_seconds: f32,
        "BattleChargeRadialEndSeconds" => pub battle_charge_radial_end_seconds: f32,
        "BattleChargeRadialThreshold" => pub battle_charge_radial_threshold: f32,
        "BattleChargeRadialLength" => pub battle_charge_radial_length: f32,
        "BattleChargeRadialIntensity" => pub battle_charge_radial_intensity: f32,
        "BattleChargeRadialFade" => pub battle_charge_radial_fade: f32,
        "BattleChargeRadialRange" => pub battle_charge_radial_range: f32,
        "BattleChargeRadialTwist" => pub battle_charge_radial_twist: f32,
        "PoseDelaySecs" => pub pose_delay_secs: f32,
        "ChargeDelay" => pub charge_delay: f32,
        "NumPeopleHit" => pub num_people_hit: Vec<i32>,
        "DragonExtraGumphRadiusMultiplier" => pub dragon_extra_gumph_radius_multiplier: f32,
        "DragonExtraGumphRadius" => pub dragon_extra_gumph_radius: f32,
    }
}
