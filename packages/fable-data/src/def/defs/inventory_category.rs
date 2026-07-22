use crate::DefStruct;
use crate::def::wire::{DefIndex, DefString};

#[derive(Debug, Clone, PartialEq, DefStruct)]
pub struct InventoryCategoryDef {
    #[def("Inventory")]
    pub inventory: DefIndex,
    #[def("NumberOfSlots")]
    pub number_of_slots: i32,
    #[def("DrawItemSlots")]
    pub draw_item_slots: bool,
    #[def("SelectEmptySlots")]
    pub select_empty_slots: bool,
    #[def("WrapHighlightCursor")]
    pub wrap_highlight_cursor: bool,
    #[def("CategoryName", default = DefString(0))]
    pub category_name: DefString,
    #[def("AllowItemsToFillMoreThanOneSlot")]
    pub allow_items_to_fill_more_than_one_slot: bool,
    #[def("CategoryIdentifier")]
    pub category_identifier: i32,
    #[def("AddCategoryOnCreate", default = true)]
    pub add_category_on_create: bool,
}
