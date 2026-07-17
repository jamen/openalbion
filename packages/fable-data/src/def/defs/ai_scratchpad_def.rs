use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CAIScratchpadDef` — C++ `CAIScratchpadDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct AIScratchpadDef {
        "ThankingPhrase" => pub thanking_phrase: u32,
        "IgnoringPhrase" => pub ignoring_phrase: u32,
        "WanderCentrePoint" => pub wander_centre_point: Vector3D,
        "WanderMinDistance" => pub wander_min_distance: f32,
        "WanderMaxDistance" => pub wander_max_distance: f32,
        "GossipCounter" => pub gossip_counter: i32,
        "MaxGossipPhrase" => pub max_gossip_phrase: i32,
        "WarningPhrase" => pub warning_phrase: u32,
        "BeerRequestPhrase" => pub beer_request_phrase: u32,
        "ScriptingStateGroup" => pub scripting_state_group: ScriptingStateGroups,
        "MaxHeroReactionDistance" => pub max_hero_reaction_distance: f32,
        "ActionFrequency" => pub action_frequency: i32,
        "ActionFrequencyVariation" => pub action_frequency_variation: f32,
        "Action" => pub action: String,
        "FaceHeroForAction" => pub face_hero_for_action: bool,
        "TargetName" => pub target_name: String,
        "FollowDistance" => pub follow_distance: f32,
        "AttackHeroOnSight" => pub attack_hero_on_sight: bool,
        "TimeToSpendHarassingHero" => pub time_to_spend_harassing_hero: i32,
        "CombatNearbyEnemyFleeingBreakOffRange" => pub combat_nearby_enemy_fleeing_break_off_range: f32,
        "CombatNearbyBreakOffRange" => pub combat_nearby_break_off_range: f32,
        "StealStealableItems" => pub steal_stealable_items: bool,
        "RecoverStealableItems" => pub recover_stealable_items: bool,
        "TakeStealableItemToRandomDestination" => pub take_stealable_item_to_random_destination: bool,
        "KillSelfAndStealableItemAfterReachingDestination" => pub kill_self_and_stealable_item_after_reaching_destination: bool,
        "AllowedToFollow" => pub allowed_to_follow: bool,
        "TableName" => pub table_name: String,
        "SeatName" => pub seat_name: String,
        "DisableHeadLooking" => pub disable_head_looking: bool,
        "IsPushableByHero" => pub is_pushable_by_hero: bool,
        "LookForFiniteTime" => pub look_for_finite_time: bool,
        "AvoidRegionExits" => pub avoid_region_exits: bool,
        "TargetingDistanceOffset" => pub targeting_distance_offset: f32,
    }
}
