use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CInventoryItemDef` | `INVENTORY_ITEM` — C++ `CInventoryItemDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct InventoryItemDef {
        "Graphic" => pub graphic: EngineGraphic,
        "ItemDescription" => pub item_description: DefString,
        "ItemDetails" => pub item_details: DefString,
        "InventoryCategory" => pub inventory_category: i32,
        "MaxNumberItems" => pub max_number_items: i32,
        "SlotIndex" => pub slot_index: i32,
        "ActivationTime" => pub activation_time: f32,
        "UseButtonAction" => pub use_button_action: i32,
        "InventoryType" => pub inventory_type: i32,
        "Orientation" => pub orientation: i32,
        "HeroAbilityDef" => pub hero_ability_def: i32,
        "IsSellable" => pub is_sellable: bool,
        "IsBuyable" => pub is_buyable: bool,
        "IsConfiscatable" => pub is_confiscatable: bool,
        "DoNotPersistUntilQuestCompleted" => pub do_not_persist_until_quest_completed: bool,
        "DoNotAutoPickUp" => pub do_not_auto_pick_up: bool,
        "AutoPickUpAfterFirstPickUp" => pub auto_pick_up_after_first_pick_up: bool,
        "ItemToSelectUponRemoval" => pub item_to_select_upon_removal: i32,
        "TutorialCategory" => pub tutorial_category: i32,
        "UIInventoryCategory" => pub ui_inventory_category: i32,
    }
}
