use crate::def_struct;

def_struct! {
    /// `PLAYER_INVENTORY` — C++ `CPlayerInventoryDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct PlayerInventoryDef {
        "Slot" => pub slot: Vec<i32>,
        "MaxInSlot" => pub max_in_slot: Vec<i32>,
        "InitialAmount" => pub initial_amount: Vec<i32>,
        "IsUsable" => pub is_usable: Vec<bool>,
        "InitialSelectedSlot" => pub initial_selected_slot: i32,
    }
}
