use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CQuestCardDef` — C++ `CQuestCardDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct QuestCardDef {
        "QuestName" => pub quest_name: i32,
        "QuestSummary" => pub quest_summary: i32,
        "QuestObjective" => pub quest_objective: i32,
        "SuccessSummary" => pub success_summary: i32,
        "RegionName" => pub region_name: DefString,
        "TeleporterRegionName" => pub teleporter_region_name: DefString,
        "InventoryCategory" => pub inventory_category: i32,
        "RenownReward" => pub renown_reward: i32,
        "GoldReward" => pub gold_reward: i32,
        "RewardObjects" => pub reward_objects: Vec<i32>,
        "IsCoreQuest" => pub is_core_quest: bool,
        "IsVignette" => pub is_vignette: bool,
        "IsExclusive" => pub is_exclusive: bool,
        "MakeVignetteRouteAppearOnMinimap" => pub make_vignette_route_appear_on_minimap: bool,
        "Prerequisites" => pub prerequisites: VecMap<i32, IdleStateGroup>,
        "NumBoasts" => pub num_boasts: i32,
        "CanPlayerCancel" => pub can_player_cancel: bool,
        "QuestEpilogue" => pub quest_epilogue: i32,
    }
}
