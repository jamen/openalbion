use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `SPECIAL_ABILITIES_LIGHTNING_SPELL_DEF` — C++ `CSpecialAbilitiesLightningSpellDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SpecialAbilitiesLightningSpellDef {
        "InitSound" => pub init_sound: DefString,
        "LoopingSound" => pub looping_sound: DefString,
        "EndHeroSound" => pub end_hero_sound: DefString,
        "EndAttackeeSound" => pub end_attackee_sound: DefString,
        "RandomBeamMinAngleX" => pub random_beam_min_angle_x: f32,
        "RandomBeamMaxAngleX" => pub random_beam_max_angle_x: f32,
        "RandomBeamMinAngleZ" => pub random_beam_min_angle_z: f32,
        "RandomBeamMaxAngleZ" => pub random_beam_max_angle_z: f32,
        "AreaEffect" => pub area_effect: Vec<f32>,
        "DamagePerHit" => pub damage_per_hit: Vec<f32>,
        "SubBeamDamagePerHit" => pub sub_beam_damage_per_hit: Vec<f32>,
        "ActivationTime" => pub activation_time: Vec<f32>,
        "DeActivationTime" => pub de_activation_time: Vec<f32>,
        "MinMainBeams" => pub min_main_beams: Vec<i32>,
        "MaxMainBeams" => pub max_main_beams: Vec<i32>,
        "MinSubBeams" => pub min_sub_beams: Vec<i32>,
        "MaxSubBeams" => pub max_sub_beams: Vec<i32>,
        "TemporaryBeamLife" => pub temporary_beam_life: Vec<f32>,
        "TargettingFOV" => pub targetting_fov: Vec<f32>,
        "StaminaPerSecond" => pub stamina_per_second: Vec<f32>,
        "MainBeamDecapitationTime" => pub main_beam_decapitation_time: Vec<f32>,
        "SubBeamDecapitationTime" => pub sub_beam_decapitation_time: Vec<f32>,
        "SubBeamDelay" => pub sub_beam_delay: Vec<f32>,
        "OnhitEveryNSecs" => pub onhit_every_n_secs: Vec<f32>,
        "CombatMultiplierIncreaseTime" => pub combat_multiplier_increase_time: f32,
        "LightningThemeTransitionInSecs" => pub lightning_theme_transition_in_secs: f32,
        "LightningThemeTransitionOutSecs" => pub lightning_theme_transition_out_secs: f32,
    }
}
