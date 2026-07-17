use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `INVENTORY_CATEGORY` — C++ `CInventoryCategoryDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct InventoryCategoryDef {
        "Inventory" => pub inventory: i32,
        "NumberOfSlots" => pub number_of_slots: i32,
        "DrawItemSlots" => pub draw_item_slots: bool,
        "SelectEmptySlots" => pub select_empty_slots: bool,
        "WrapHighlightCursor" => pub wrap_highlight_cursor: bool,
        "CategoryName" => pub category_name: DefString,
        "AllowItemsToFillMoreThanOneSlot" => pub allow_items_to_fill_more_than_one_slot: bool,
        "CategoryIdentifier" => pub category_identifier: i32,
        "AddCategoryOnCreate" => pub add_category_on_create: bool,
    }
}
