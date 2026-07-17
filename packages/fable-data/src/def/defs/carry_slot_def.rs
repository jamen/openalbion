use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CARRY_SLOT` — C++ `CCarrySlotDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CarrySlotDef {
        "DummyPosName" => pub dummy_pos_name: DefString,
        "DummyPosIndex" => pub dummy_pos_index: i32,
        "PrimarySlot" => pub primary_slot: bool,
        "SecondaryDummyPosName" => pub secondary_dummy_pos_name: DefString,
    }
}
