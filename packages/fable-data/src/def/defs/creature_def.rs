use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCreatureDef` — C++ `CCreatureDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CreatureDef {
        "WoundMorphs" => pub wound_morphs: WoundMorphs,
        "RandomAppearanceMorph" => pub random_appearance_morph: RandomAppearanceMorph,
        "Expressions" => pub expressions: ExpressionSet,
        "Stats" => pub stats: CreatureStatsDef,
        "InitialActions" => pub initial_actions: Vec<i32>,
        "InitialAppearanceModifiers" => pub initial_appearance_modifiers: Vec<i32>,
        "PhonemeAnim" => pub phoneme_anim: VecMap<String, i32>,
        "DialogueVoices" => pub dialogue_voices: Vec<DefString>,
        "Inventory" => pub inventory: i32,
        "ClothingInventory" => pub clothing_inventory: i32,
        "WeaponsInventory" => pub weapons_inventory: i32,
        "HeroAbilitiesScreenInventory" => pub hero_abilities_screen_inventory: i32,
        "QuestCardScreenInventory" => pub quest_card_screen_inventory: i32,
        "MapScreenInventory" => pub map_screen_inventory: i32,
        "MagicScreenInventory" => pub magic_screen_inventory: i32,
        "StatsScreenInventory" => pub stats_screen_inventory: i32,
        "ExperienceScreenInventory" => pub experience_screen_inventory: i32,
        "TradeScreenInventory" => pub trade_screen_inventory: i32,
        "AugmentScreenInventory" => pub augment_screen_inventory: i32,
        "QuestsScreenInventory" => pub quests_screen_inventory: i32,
        "InGameMenuScreen" => pub in_game_menu_screen: i32,
        "CreatureGroup" => pub creature_group: u32,
        "BattleCrySound" => pub battle_cry_sound: DefString,
        "OpinionSourceDef" => pub opinion_source_def: i32,
        "RespawnWaitInDaysMin" => pub respawn_wait_in_days_min: i32,
        "RespawnWaitInDaysMax" => pub respawn_wait_in_days_max: i32,
        "RespawnWaitInFramesMin" => pub respawn_wait_in_frames_min: i32,
        "RespawnWaitInFramesMax" => pub respawn_wait_in_frames_max: i32,
        "UseActualHitPosForHitEffects" => pub use_actual_hit_pos_for_hit_effects: bool,
        "Short" => pub short: bool,
        "FlashOnHit" => pub flash_on_hit: bool,
        "PickPocketable" => pub pick_pocketable: bool,
        "ChildThatCanRegionFollow" => pub child_that_can_region_follow: bool,
        "AllowedInProtectedTowns" => pub allowed_in_protected_towns: bool = true,
        "RadiusWeight" => pub radius_weight: VecMap<Opinion, f32>,
        "RadiusSpread" => pub radius_spread: FloatRange,
        "AGhostIsWhatIBe" => pub a_ghost_is_what_i_be: bool,
    }
}
