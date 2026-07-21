use crate::def_struct;

def_struct! {
    /// `COMBAT_DIALOGUE_DEF` — C++ `CCombatDialogueDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CombatDialogueDef {
        "MinTimeBetweenSpeech" => pub min_time_between_speech: f32,
        "MinTimeBetweenSpeechRepeat" => pub min_time_between_speech_repeat: f32,
        "MinDelayAfterBattleCry" => pub min_delay_after_battle_cry: f32,
        "EnemySighted" => pub enemy_sighted: i32,
        "RunForCover" => pub run_for_cover: i32,
        "Hiding" => pub hiding: i32,
        "Ambushing" => pub ambushing: i32,
        "WillSeenUsed" => pub will_seen_used: i32,
        "Threat" => pub threat: i32,
        "Encourage" => pub encourage: i32,
        "FoeKnockedDownByColleague" => pub foe_knocked_down_by_colleague: i32,
        "MenLosing" => pub men_losing: i32,
        "EnemyKnockedDown" => pub enemy_knocked_down: i32,
        "RangedCombatAiming" => pub ranged_combat_aiming: i32,
        "RangedCombatAimingBlocked" => pub ranged_combat_aiming_blocked: i32,
        "Injured" => pub injured: i32,
        "ReactionToDecapitation" => pub reaction_to_decapitation: i32,
        "ColleagueInjured" => pub colleague_injured: i32,
        "Scared" => pub scared: i32,
        "Flee" => pub flee: i32,
    }
}
