use crate::def_struct;
use crate::def::prelude::*;

def_struct! {
    /// `CCarryableDef` — C++ `CCarryableDef`.
    #[derive(Debug, Clone, PartialEq)]
    pub struct CarryableDef {
        "ActiveCarrySlot" => pub active_carry_slot: i32,
        "SecondaryActiveCarrySlot" => pub secondary_active_carry_slot: i32,
        "PassiveCarrySlot" => pub passive_carry_slot: i32,
        "IncludeInThingScansWhenCarried" => pub include_in_thing_scans_when_carried: bool,
        "OnKillFX" => pub on_kill_fx: i32,
        "OffsetCoordRelativeToAttachToDummy" => pub offset_coord_relative_to_attach_to_dummy: Vector3D,
        "OffsetAmountForThingsOnTopOfMe" => pub offset_amount_for_things_on_top_of_me: f32,
        "PassiveCarrySlotScale" => pub passive_carry_slot_scale: f32 = 1.0,
    }
}
